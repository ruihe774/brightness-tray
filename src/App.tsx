import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

export default function App() {
  const [monitors, setMonitors] = useState([] as {id: string, name: string | null, maximum: number, current: number}[]);

  const refreshHandler = useCallback(() => {
    invoke("refresh_monitors");
  }, [])

  const listHandler = useCallback(async () => {
    const ids: string[] = await invoke("get_monitors");
    const monitors = [];
    for (const id of ids) {
      let name: string | null = await invoke("get_monitor_user_friendly_name", {id});
      let reply: {current: number, maximum: number} = await invoke("get_monitor_feature", {id, feature: "luminance"})
      monitors.push({id, name, ...reply});
    }
    setMonitors(monitors);
  }, [])

  const changeHandle = useCallback(async (e: React.FormEvent) => {
    const target = e.target as HTMLInputElement;
    const id = target.dataset.monitorId;
    const value = Number(target.value);
    await invoke("set_monitor_feature", {id, feature: "luminance", value});
    await listHandler()
  }, []);

  return (
    <div className="container">
      <button type="button" onClick={refreshHandler}>Refresh</button>
      <button type="button" onClick={listHandler}>List</button>
      <ul>
        {monitors.map(monitor => (
          <li key={monitor.id}>
            <div><b>{monitor.name}</b>: <code>{monitor.id}</code></div>
            <div>
              <input type="range" min="0" max={monitor.maximum} value={monitor.current} onChange={changeHandle} data-monitor-id={monitor.id} />
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
};
