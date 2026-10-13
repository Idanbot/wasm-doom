declare const __BLACKSITE_UPDATED_AT__: string;

export const LATEST_UPDATE_DATE = __BLACKSITE_UPDATED_AT__;
export const LATEST_UPDATE_LABEL = new Intl.DateTimeFormat("en-GB", {
  day: "numeric", month: "short", year: "numeric", timeZone: "UTC",
}).format(new Date(`${LATEST_UPDATE_DATE}T00:00:00Z`));
