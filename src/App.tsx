import { useCallback, useEffect, useState, Fragment } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";


type Shortcuts = {
  AppName: string,
  appid: number,
  icon: string
  LastPlayTime: number
}

type UserShortcuts = Record<string, Shortcuts>
type AllShortcuts = Record<string, UserShortcuts>

// type fun = Record<String, >

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

  if (!allShortcuts)
    return;

  console.log(allShortcuts)
  return (
    <div className="column-container">
      {Object.keys(allShortcuts).map(accountIdKey =>
        <Fragment key={accountIdKey}>
          <div className="id-header" key={`header-${accountIdKey}`}>
            Account {accountIdKey}
          </div>

          {Object.values(allShortcuts[accountIdKey]).map(entry =>
            <div className="shortcut-entry" key={entry.appid}>
              <img src={entry.icon} />
              {entry.AppName}
            </div>
          )}
        </Fragment>
      )}
    </div>
  );
}

export default App;
