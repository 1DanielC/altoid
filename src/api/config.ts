// API environment configuration matching Rust ApiEnv
export const API_HOSTS = {
  local: 'http://localhost:8080',
  can: 'https://can.openspace.ai',
  eu: 'https://eu.openspace.ai',
  gov: 'https://gov.openspace.ai',
  jpn: 'https://jpn.openspace.ai',
  ksa: 'https://ksa.openspace.ai',
  uk: 'https://uk.openspace.ai',
  us: 'https://openspace.ai',
  sgp: 'https://sgp.openspace.ai',
} as const;

export type ApiEnvironment = keyof typeof API_HOSTS;
