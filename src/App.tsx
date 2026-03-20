import { getCurrentWindow } from "@tauri-apps/api/window";
import { Dropdown } from "./pages/Dropdown";
import { Dashboard } from "./pages/Dashboard";
import "./App.css";

const windowLabel = getCurrentWindow().label;

function App() {
  if (windowLabel === "dashboard") return <Dashboard />;
  return <Dropdown />;
}

export default App;
