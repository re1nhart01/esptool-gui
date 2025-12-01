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
  newInvalidation: () => void;
  updateConfig: (new_cfg: Config) => Promise<void>;
}

export const ConfigContext = createContext<ConfigContextType | null>(null);

export const ConfigContextProvider: FC<PropsWithChildren> = ({ children }) => {
  const [value, setValue] = useState<Config | null>(null);
  const [invalidate, setInvalidate] = useState(0);
  const PATH = "";

  const newInvalidation = () => {
    setInvalidate((prev) => prev + 1);
  };

  const updateConfig = async (newCfg: Config) => {
    try {
      const isUpdated = await invoke("tauri_update_config_data", {
        newCfg,
        PATH,
      });
      if (isUpdated) {
        setValue(newCfg);
      }
    } catch (e) {
      console.log(e);
    }
  };

  const contextValue = { value, setValue, newInvalidation, updateConfig };

  useEffect(() => {
    const handle = async () => {
      try {
        const data: Config = await invoke("tauri_get_config_data");
        console.log(data);
        setValue(data);
      } catch (e) {
        console.log(e);
      }
    };

    handle().then();
  }, [invalidate]);

  return (
    <ConfigContext.Provider value={contextValue}>
      {children}
    </ConfigContext.Provider>
  );
};
