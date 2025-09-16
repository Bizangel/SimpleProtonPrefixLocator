import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";


type ProtonApp = {
  appname: string,
  appid: string,
  icon: string,
  lastplaytime: string,
  exe: string,
  startdir: string,
}

function App() {
  const [allApps, setAllApps] = useState<ProtonApp[]>([]);
  const [errorMsg, setErrorMsg] = useState<string>("");

  const readSteamVdfShortcuts = useCallback(async () => {
    const res: any = await invoke("read_steam_vdf_shortcuts");
    const parsed  = JSON.parse(res);
    if ('error' in parsed) {
      setErrorMsg(parsed["error"]);
      return;
    }

    setAllApps(parsed);
  }, [setAllApps, setErrorMsg])

  useEffect(() => {
    readSteamVdfShortcuts();
  }, [setAllApps])

  const openAppIdPrefix = useCallback(async (appid: string) => {
    await invoke("open_appid_prefix", {appid: appid });
  }, [])

  if (errorMsg)
    return (
      <div className="error-container">
        <h1>Error reading local steam info</h1>
        {errorMsg}
      </div>
    )

  return (
    <div className="column-container">
      {allApps.map(appEntry =>
          <div className="shortcut-entry" key={appEntry.appid} onClick={() => { openAppIdPrefix(appEntry.appid) }}>
            <div className={`shortcut-img-container ${appEntry.icon ? "" : "noimg"}`}>
              {appEntry.icon && <img src={appEntry.icon} className="shortcut-img" />}
            </div>
            <div className="shortcut-entry-title">
                <b>App ID: { appEntry.appid} </b>
                <h3>{ appEntry.appname}</h3>
            </div>

            { appEntry.exe &&
              <div className="shortcut-entry-path">
                  Shortcut<br/>{appEntry.exe}
              </div>
            }
          </div>
      )}
    </div>
  );
}

export default App;
