import { Config } from "@/types/types";
import { invoke } from "@tauri-apps/api/core";
import {
  createContext,
  FC,
  PropsWithChildren,
  useEffect,
  useState,
} from "react";

export interface ConfigContextType {
  value: Config | null;
  setValue: React.Dispatch<React.SetStateAction<Config | null>>;
  updateConfig: (new_cfg: Config | null) => Promise<void>;
  getInitialConfig: () => Promise<void>;
}

export const ConfigContext = createContext<ConfigContextType | null>(null);

export const ConfigContextProvider: FC<PropsWithChildren> = ({ children }) => {
  const [value, setValue] = useState<Config | null>(null);

  const getInitialConfig = async () => {
    try {
      const data: Config = await invoke("tauri_get_config_data");
      console.log(data);
      setValue(data);
    } catch (e) {
      console.log(e);
    }
  };

  const updateConfig = async (newCfg: Config | null) => {
    try {
      const isUpdated = await invoke("tauri_update_config_data", {
        newCfg,
      });
      if (isUpdated) {
        await getInitialConfig();
      }
    } catch (e) {
      console.log(e);
    }
  };

  const contextValue = { value, setValue, updateConfig, getInitialConfig };

  useEffect(() => {
    getInitialConfig().then();
  }, []);

  return (
    <ConfigContext.Provider value={contextValue}>
      {children}
    </ConfigContext.Provider>
  );
};
