import { useState } from "react";
import reactLogo from "./assets/react.svg";
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
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const res: any = await invoke("read_steam_vdf_shortcuts");
    const parsed  = JSON.parse(res);
    if ('error' in parsed) {
      const errormsg = parsed["error"]
      // TODO: error handling
      console.log("Error: ", errormsg)
      return;
    }

    const vdfTypes: AllShortcuts = parsed;
    // TODO: use display type etc.
  }

  return (
    <main className="container">
      <button onClick={greet}>
        read vdf
      </button>

      <div className="entry">
        Hello
      </div>
      <div className="entry">
        Hello
      </div>
    </main>
  );
}

export default App;
