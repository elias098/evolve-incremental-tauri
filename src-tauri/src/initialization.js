// import { appWindow } from "@tauri-apps/api/window";
// import { invoke } from "@tauri-apps/api/tauri";
// import { confirm } from "@tauri-apps/api/dialog";

// const { appWindow } = await import("@tauri-apps/api/window");
// const { invoke } = await import("@tauri-apps/api/tauri");
// const { confirm } = await import("@tauri-apps/api/dialog");

setTimeout(async () => {
  const { appWindow } = window.__TAURI__.window;
  const { invoke } = window.__TAURI__.tauri;
  const { message, confirm } = window.__TAURI__.dialog;

  document.addEventListener(
    "click",
    (event) => {
      if (
        event.target instanceof HTMLAnchorElement &&
        event.target.tagName !== "a" &&
        event.target.href !== ""
      ) {
        event.stopPropagation();

        invoke("open_window", { href: event.target.href }).catch((err) =>
          message(err)
        );
      }
    },
    true
  );

  function save() {
    let save_string = window.exportGame();
    let save_string_json = LZString.decompressFromBase64(save_string);
    let save = JSON.parse(save_string_json);
    return invoke("save", {
      fileName: "reset-" + save.stats.reset.toString().padStart(3, "0"),
      saveData: save_string,
    });
  }

  function save_UTF16(export_string_utf16) {
    const save_string_json = LZString.decompressFromUTF16(export_string_utf16);
    const save_string = LZString.compressToBase64(save_string_json);
    const save = JSON.parse(save_string_json);

    return invoke("save", {
      fileName: "reset-" + save.stats.reset.toString().padStart(3, "0"),
      saveData: save_string,
    });
  }

  function save_script_settings() {
    return invoke("save_script_settings", {
      scriptSettings: localStorage.getItem("settings"),
    });
  }

  function log_action(action) {
    return invoke("log_action", {
      fileName:
        "reset-" + evolve.global.stats.reset.toString().padStart(3, "0"),
      action: action,
    });
  }

  // Save to file on exit
  const unlisten = await appWindow.onCloseRequested(async (event) => {
    let promises = Promise.allSettled([save(), save_script_settings()]);
    let rejections = (await promises).filter(
      (promise) => promise.status == "rejected"
    );

    if (rejections.length !== 0) {
      let rejection_messages = rejections
        .map((rejection) => rejection.reason)
        .join("\n");
      const confirmed = await confirm(
        `${rejection_messages}\n\nAre you sure you want to exit?`
      );

      if (!confirmed) {
        event.preventDefault();
      }
    }
  });

  // Save to file when the game saves
  const localStore = localStorage.setItem;
  let last_save = 0;
  localStorage.setItem = function (key, value) {
    if (key === "evolved" && Date.now() > last_save + 300 * 1000) {
      last_save = Date.now();
      save_UTF16(value).catch((err) => message(err));
    }

    if (key === "evolveBak") {
      save_UTF16(value).catch((err) => message(err));
      save_script_settings().catch((err) => message(err));
    }

    localStore.apply(this, arguments);
  };

  // Modify unshift for the message arrays to update the log on change
  function listen_unshift(type) {
    document.getElementById("msgQueue").__vue__.m[type].unshift = function () {
      for (const message of arguments) {
        const days = evolve.global.stats.days.toString().padStart(4);
        const prefix = `[${type}]`.padStart(16);
        log_action(`day ${days}: ${prefix} ${message.msg}`).catch((err) =>
          message(err)
        );
      }
      return Array.prototype.unshift.apply(this, arguments);
    };
  }

  listen_unshift("building_queue");
  listen_unshift("research_queue");
  listen_unshift("progress");
  listen_unshift("achievements");

  const old_importGame = window.importGame;
  window.importGame = function () {
    invoke("log_action", {
      fileName:
        "reset-" + evolve.global.stats.reset.toString().padStart(3, "0"),
      day: evolve.global.stats.days,
      action: "\nImporting save\n",
    }).catch((err) => message(err));
    return old_importGame.apply(null, arguments);
  };

  const old_soft_reset = window.soft_reset;
  window.soft_reset = function () {
    invoke("clear_log", {
      fileName:
        "reset-" + evolve.global.stats.reset.toString().padStart(3, "0"),
    }).catch((err) => message(err));
    return old_soft_reset.apply(null, arguments);
  };
}, 1000);
