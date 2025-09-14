import { useCallback, useEffect, useState, Fragment } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";


type Shortcuts = {
  appname: string,
  appid: number,
  icon: string,
  lastplaytime: number,
  exe: string,
  startdir: string,
}

type UserShortcuts = Record<string, Shortcuts>
type AllShortcuts = Record<string, UserShortcuts>

function App() {
  const [allShortcuts, setAllShortcuts] = useState<AllShortcuts>();
  const [errorMsg, setErrorMsg] = useState<string>("");

  const readSteamVdfShortcuts = useCallback(async () => {
    const res: any = await invoke("read_steam_vdf_shortcuts");
    const parsed  = JSON.parse(res);
    if ('error' in parsed) {
      setErrorMsg(parsed["error"]);
      return;
    }

    const vdfTypes: AllShortcuts = parsed;
    setAllShortcuts(vdfTypes);
  }, [setAllShortcuts, setErrorMsg])

  useEffect(() => {
    readSteamVdfShortcuts();
  }, [setAllShortcuts])

  const openAppIdPrefix = useCallback(async (appid: number, userid: string) => {
    await invoke("open_appid_prefix", {appid: appid.toString(), userid: userid });
  }, [])

  if (!allShortcuts)
    return;

  console.log(allShortcuts)
  if (errorMsg)
    return (
      <div className="error-container">
        <h1>Error reading local steam info</h1>
        {errorMsg}
      </div>
    )

  return (
    <div className="column-container">
      {Object.keys(allShortcuts).map(accountIdKey =>
        <Fragment key={accountIdKey}>
          <div className="id-header" key={`header-${accountIdKey}`}>
            Account {accountIdKey}
          </div>

          {[...Object.values(allShortcuts[accountIdKey])].sort((a,b) => a.lastplaytime - b.lastplaytime).map(entry =>
            <div className="shortcut-entry" key={entry.appid} onClick={() => {openAppIdPrefix(entry.appid, accountIdKey)}}>
              {entry.icon && <img src={entry.icon} className="shortcut-img" />}
              <div className="shortcut-entry-title">
                 <b>App ID: { entry.appid} </b>
                 <h1>{ entry.appname}</h1>
              </div>
              <div className="shortcut-entry-path">
                 {entry.exe}
              </div>
            </div>
          )}
        </Fragment>
      )}
    </div>
  );
}

export default App;
