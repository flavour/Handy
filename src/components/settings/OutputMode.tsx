import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dropdown } from "../ui/Dropdown";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";

import type { OutputMode } from "@/bindings";

interface OutputModeProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const OutputModeSetting: React.FC<OutputModeProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const outputMode = (getSetting("output_mode") || "paste") as OutputMode;

    const options = [
      {
        value: "paste",
        label: t("settings.advanced.outputMode.options.paste"),
      },
      {
        value: "opencode",
        label: t("settings.advanced.outputMode.options.opencode"),
      },
      {
        value: "openclaw",
        label: t("settings.advanced.outputMode.options.openclaw"),
      },
      {
        value: "discord",
        label: t("settings.advanced.outputMode.options.discord"),
      },
      {
        value: "matrix",
        label: t("settings.advanced.outputMode.options.matrix"),
      },
    ];

    const opencodeBaseUrl = (getSetting("opencode_base_url") ?? "") as string;
    const openclawBaseUrl = (getSetting("openclaw_base_url") ?? "") as string;
    const openclawToken = (getSetting("openclaw_token") ?? "") as string;
    const openclawSessionKey = (getSetting("openclaw_session_key") ??
      "") as string;
    const discordBotToken = (getSetting("discord_bot_token") ?? "") as string;
    const discordChannelId = (getSetting("discord_channel_id") ?? "") as string;
    const matrixHomeserverUrl = (getSetting("matrix_homeserver_url") ??
      "") as string;
    const matrixAccessToken = (getSetting("matrix_access_token") ??
      "") as string;
    const matrixRoomId = (getSetting("matrix_room_id") ?? "") as string;

    const [localOpencodeBaseUrl, setLocalOpencodeBaseUrl] =
      useState(opencodeBaseUrl);
    const [localOpenclawBaseUrl, setLocalOpenclawBaseUrl] =
      useState(openclawBaseUrl);
    const [localOpenclawToken, setLocalOpenclawToken] = useState(openclawToken);
    const [localOpenclawSessionKey, setLocalOpenclawSessionKey] =
      useState(openclawSessionKey);
    const [localDiscordBotToken, setLocalDiscordBotToken] =
      useState(discordBotToken);
    const [localDiscordChannelId, setLocalDiscordChannelId] =
      useState(discordChannelId);
    const [localMatrixHomeserverUrl, setLocalMatrixHomeserverUrl] =
      useState(matrixHomeserverUrl);
    const [localMatrixAccessToken, setLocalMatrixAccessToken] =
      useState(matrixAccessToken);
    const [localMatrixRoomId, setLocalMatrixRoomId] = useState(matrixRoomId);

    useEffect(() => {
      setLocalOpencodeBaseUrl(opencodeBaseUrl);
    }, [opencodeBaseUrl]);

    useEffect(() => {
      setLocalOpenclawBaseUrl(openclawBaseUrl);
    }, [openclawBaseUrl]);

    useEffect(() => {
      setLocalOpenclawToken(openclawToken);
    }, [openclawToken]);

    useEffect(() => {
      setLocalOpenclawSessionKey(openclawSessionKey);
    }, [openclawSessionKey]);

    useEffect(() => {
      setLocalDiscordBotToken(discordBotToken);
    }, [discordBotToken]);

    useEffect(() => {
      setLocalDiscordChannelId(discordChannelId);
    }, [discordChannelId]);

    useEffect(() => {
      setLocalMatrixHomeserverUrl(matrixHomeserverUrl);
    }, [matrixHomeserverUrl]);

    useEffect(() => {
      setLocalMatrixAccessToken(matrixAccessToken);
    }, [matrixAccessToken]);

    useEffect(() => {
      setLocalMatrixRoomId(matrixRoomId);
    }, [matrixRoomId]);

    const outputModeBusy = isUpdating("output_mode");

    const opencodeBusy = isUpdating("opencode_base_url") || outputModeBusy;
    const openclawBusy =
      isUpdating("openclaw_base_url") ||
      isUpdating("openclaw_token") ||
      isUpdating("openclaw_session_key") ||
      outputModeBusy;
    const discordBusy =
      isUpdating("discord_bot_token") ||
      isUpdating("discord_channel_id") ||
      outputModeBusy;
    const matrixBusy =
      isUpdating("matrix_homeserver_url") ||
      isUpdating("matrix_access_token") ||
      isUpdating("matrix_room_id") ||
      outputModeBusy;

    // Persist config fields even if the user doesn't blur the input.
    // This avoids a race where the UI shows a value but transcription reads the old settings.
    const debounceMs = 500;

    useEffect(() => {
      if (outputMode !== "opencode") return;
      if (localOpencodeBaseUrl === opencodeBaseUrl) return;
      const t = setTimeout(() => {
        void updateSetting("opencode_base_url", localOpencodeBaseUrl);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localOpencodeBaseUrl,
      opencodeBaseUrl,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "openclaw") return;
      if (localOpenclawBaseUrl === openclawBaseUrl) return;
      const t = setTimeout(() => {
        void updateSetting("openclaw_base_url", localOpenclawBaseUrl);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localOpenclawBaseUrl,
      openclawBaseUrl,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "openclaw") return;
      if (localOpenclawToken === openclawToken) return;
      const t = setTimeout(() => {
        void updateSetting("openclaw_token", localOpenclawToken);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localOpenclawToken,
      openclawToken,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "openclaw") return;
      if (localOpenclawSessionKey === openclawSessionKey) return;
      const t = setTimeout(() => {
        void updateSetting("openclaw_session_key", localOpenclawSessionKey);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localOpenclawSessionKey,
      openclawSessionKey,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "discord") return;
      if (localDiscordBotToken === discordBotToken) return;
      const t = setTimeout(() => {
        void updateSetting("discord_bot_token", localDiscordBotToken);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localDiscordBotToken,
      discordBotToken,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "discord") return;
      if (localDiscordChannelId === discordChannelId) return;
      const t = setTimeout(() => {
        void updateSetting("discord_channel_id", localDiscordChannelId);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localDiscordChannelId,
      discordChannelId,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "matrix") return;
      if (localMatrixHomeserverUrl === matrixHomeserverUrl) return;
      const t = setTimeout(() => {
        void updateSetting("matrix_homeserver_url", localMatrixHomeserverUrl);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localMatrixHomeserverUrl,
      matrixHomeserverUrl,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "matrix") return;
      if (localMatrixAccessToken === matrixAccessToken) return;
      const t = setTimeout(() => {
        void updateSetting("matrix_access_token", localMatrixAccessToken);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localMatrixAccessToken,
      matrixAccessToken,
      updateSetting,
      debounceMs,
    ]);

    useEffect(() => {
      if (outputMode !== "matrix") return;
      if (localMatrixRoomId === matrixRoomId) return;
      const t = setTimeout(() => {
        void updateSetting("matrix_room_id", localMatrixRoomId);
      }, debounceMs);
      return () => clearTimeout(t);
    }, [
      outputMode,
      localMatrixRoomId,
      matrixRoomId,
      updateSetting,
      debounceMs,
    ]);

    return (
      <>
        <SettingContainer
          title={t("settings.advanced.outputMode.title")}
          description={t("settings.advanced.outputMode.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        >
          <Dropdown
            options={options}
            selectedValue={outputMode}
            onSelect={(value) =>
              updateSetting("output_mode", value as OutputMode)
            }
            disabled={outputModeBusy}
          />
        </SettingContainer>

        {outputMode === "opencode" && (
          <SettingContainer
            title={t("settings.advanced.outputMode.opencodeBaseUrl.title")}
            description={t(
              "settings.advanced.outputMode.opencodeBaseUrl.description",
            )}
            descriptionMode={descriptionMode}
            grouped={grouped}
          >
            <Input
              type="text"
              value={localOpencodeBaseUrl}
              onChange={(e) => setLocalOpencodeBaseUrl(e.target.value)}
              onBlur={() =>
                updateSetting("opencode_base_url", localOpencodeBaseUrl)
              }
              placeholder={t(
                "settings.advanced.outputMode.opencodeBaseUrl.placeholder",
              )}
              variant="compact"
              disabled={opencodeBusy}
              className="min-w-[360px]"
            />
          </SettingContainer>
        )}

        {outputMode === "openclaw" && (
          <>
            <SettingContainer
              title={t("settings.advanced.outputMode.openclawBaseUrl.title")}
              description={t(
                "settings.advanced.outputMode.openclawBaseUrl.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="text"
                value={localOpenclawBaseUrl}
                onChange={(e) => setLocalOpenclawBaseUrl(e.target.value)}
                onBlur={() =>
                  updateSetting("openclaw_base_url", localOpenclawBaseUrl)
                }
                placeholder={t(
                  "settings.advanced.outputMode.openclawBaseUrl.placeholder",
                )}
                variant="compact"
                disabled={openclawBusy}
                className="min-w-[360px]"
              />
            </SettingContainer>

            <SettingContainer
              title={t("settings.advanced.outputMode.openclawToken.title")}
              description={t(
                "settings.advanced.outputMode.openclawToken.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="password"
                value={localOpenclawToken}
                onChange={(e) => setLocalOpenclawToken(e.target.value)}
                onBlur={() =>
                  updateSetting("openclaw_token", localOpenclawToken)
                }
                placeholder={t(
                  "settings.advanced.outputMode.openclawToken.placeholder",
                )}
                variant="compact"
                disabled={openclawBusy}
                className="min-w-[320px]"
              />
            </SettingContainer>

            <SettingContainer
              title={t("settings.advanced.outputMode.openclawSessionKey.title")}
              description={t(
                "settings.advanced.outputMode.openclawSessionKey.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="text"
                value={localOpenclawSessionKey}
                onChange={(e) => setLocalOpenclawSessionKey(e.target.value)}
                onBlur={() =>
                  updateSetting("openclaw_session_key", localOpenclawSessionKey)
                }
                placeholder={t(
                  "settings.advanced.outputMode.openclawSessionKey.placeholder",
                )}
                variant="compact"
                disabled={openclawBusy}
                className="min-w-[420px]"
              />
            </SettingContainer>
          </>
        )}

        {outputMode === "discord" && (
          <>
            <SettingContainer
              title={t("settings.advanced.outputMode.discordBotToken.title")}
              description={t(
                "settings.advanced.outputMode.discordBotToken.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="password"
                value={localDiscordBotToken}
                onChange={(e) => setLocalDiscordBotToken(e.target.value)}
                onBlur={() =>
                  updateSetting("discord_bot_token", localDiscordBotToken)
                }
                placeholder={t(
                  "settings.advanced.outputMode.discordBotToken.placeholder",
                )}
                variant="compact"
                disabled={discordBusy}
                className="min-w-[320px]"
              />
            </SettingContainer>

            <SettingContainer
              title={t("settings.advanced.outputMode.discordChannelId.title")}
              description={t(
                "settings.advanced.outputMode.discordChannelId.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="text"
                value={localDiscordChannelId}
                onChange={(e) => setLocalDiscordChannelId(e.target.value)}
                onBlur={() =>
                  updateSetting("discord_channel_id", localDiscordChannelId)
                }
                placeholder={t(
                  "settings.advanced.outputMode.discordChannelId.placeholder",
                )}
                variant="compact"
                disabled={discordBusy}
                className="min-w-[320px]"
              />
            </SettingContainer>
          </>
        )}

        {outputMode === "matrix" && (
          <>
            <SettingContainer
              title={t(
                "settings.advanced.outputMode.matrixHomeserverUrl.title",
              )}
              description={t(
                "settings.advanced.outputMode.matrixHomeserverUrl.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="text"
                value={localMatrixHomeserverUrl}
                onChange={(e) => setLocalMatrixHomeserverUrl(e.target.value)}
                onBlur={() =>
                  updateSetting(
                    "matrix_homeserver_url",
                    localMatrixHomeserverUrl,
                  )
                }
                placeholder={t(
                  "settings.advanced.outputMode.matrixHomeserverUrl.placeholder",
                )}
                variant="compact"
                disabled={matrixBusy}
                className="min-w-[360px]"
              />
            </SettingContainer>

            <SettingContainer
              title={t("settings.advanced.outputMode.matrixRoomId.title")}
              description={t(
                "settings.advanced.outputMode.matrixRoomId.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="text"
                value={localMatrixRoomId}
                onChange={(e) => setLocalMatrixRoomId(e.target.value)}
                onBlur={() =>
                  updateSetting("matrix_room_id", localMatrixRoomId)
                }
                placeholder={t(
                  "settings.advanced.outputMode.matrixRoomId.placeholder",
                )}
                variant="compact"
                disabled={matrixBusy}
                className="min-w-[360px]"
              />
            </SettingContainer>

            <SettingContainer
              title={t("settings.advanced.outputMode.matrixAccessToken.title")}
              description={t(
                "settings.advanced.outputMode.matrixAccessToken.description",
              )}
              descriptionMode={descriptionMode}
              grouped={grouped}
            >
              <Input
                type="password"
                value={localMatrixAccessToken}
                onChange={(e) => setLocalMatrixAccessToken(e.target.value)}
                onBlur={() =>
                  updateSetting("matrix_access_token", localMatrixAccessToken)
                }
                placeholder={t(
                  "settings.advanced.outputMode.matrixAccessToken.placeholder",
                )}
                variant="compact"
                disabled={matrixBusy}
                className="min-w-[320px]"
              />
            </SettingContainer>
          </>
        )}
      </>
    );
  },
);

OutputModeSetting.displayName = "OutputModeSetting";
