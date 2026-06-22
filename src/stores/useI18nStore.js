import { create } from "zustand";
import { enUS } from "../i18n/en-US.js";
import { zhCN } from "../i18n/zh-CN.js";

const DICTS = {
  "en-US": enUS,
  "zh-CN": zhCN,
};

export const useI18nStore = create((set, get) => ({
  language: "en-US",

  setLanguage: (lang) => {
    set({ language: lang });
    document.documentElement.lang = lang;
  },

  /** Translate a key. Supports %s placeholder. */
  t: (key, ...args) => {
    const { language } = get();
    const dict = DICTS[language] || enUS;
    let s = dict[key] ?? key;
    // Replace %s placeholders
    args.forEach((arg, i) => {
      s = s.replace("%s", arg);
    });
    return s;
  },
}));