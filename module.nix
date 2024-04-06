{ config, pkgs, lib, ... }:

let
  cfg = config.services.bszet-mind;
in
{
  options.services.bszet-mind = {
    enable = lib.mkEnableOption "bszet-mind";

    package = lib.mkPackageOption pkgs "bszet-mind" { };

    listen = {
      addr = lib.mkOption {
        type = lib.types.str;
        default = "::";
      };
      port = lib.mkOption {
        type = lib.types.port;
        default = null;
      };
    };
    internalListen = {
      addr = lib.mkOption {
        type = lib.types.str;
        default = "::1";
      };
      port = lib.mkOption {
        type = lib.types.port;
        default = null;
      };
    };

    entrypoint = lib.mkOption {
      type = lib.types.str;
      default = "https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_001.html";
    };
    usernameFile = lib.mkOption {
      type = lib.types.str;
    };
    passwordFile = lib.mkOption {
      type = lib.types.str;
    };

    telegram = {
      tokenFile = lib.mkOption {
        type = lib.types.str;
      };
      chatIds = lib.mkOption {
        type = lib.types.listOf lib.types.int;
      };
    };

    apiTokenFile = lib.mkOption {
      type = lib.types.str;
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.services = {
      bszet-mind-geckodriver = {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];

        serviceConfig = {
          ExecStart = "${pkgs.geckodriver}/bin/geckodriver --binary=${pkgs.firefox}/bin/firefox --host ${if (lib.hasInfix ":" cfg.internalListen.addr) then "[${cfg.internalListen.addr}]" else cfg.internalListen.addr} --allow-hosts localhost";
          DynamicUser = true;
          User = "bszet-mind-geckodriver";
        };
      };

      bszet-mind = {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" "bszet-mind-geckodriver.service" ];

        environment = {
          BSZET_MIND_ENTRYPOINT = cfg.entrypoint;
          BSZET_MIND_USERNAME_FILE = "%d/username";
          BSZET_MIND_PASSWORD_FILE = "%d/password";
          BSZET_MIND_TELEGRAM_TOKEN_FILE = "%d/telegram_token";
          BSZET_MIND_CHAT_IDS = builtins.concatStringsSep "," (map (id: builtins.toString id) cfg.telegram.chatIds);
          BSZET_MIND_GECKO_DRIVER_URl = "http://${if (lib.hasInfix ":" cfg.internalListen.addr) then "[${cfg.internalListen.addr}]" else cfg.internalListen.addr}:4444";
          BSZET_MIND_LISTEN_ADDR = "${if (lib.hasInfix ":" cfg.listen.addr) then "[${cfg.listen.addr}]" else cfg.listen.addr}:${toString cfg.listen.port}";
          BSZET_MIND_INTERNAL_LISTEN_ADDR = "${if (lib.hasInfix ":" cfg.internalListen.addr) then "[${cfg.internalListen.addr}]" else cfg.internalListen.addr}:${toString cfg.internalListen.port}";
          BSZET_MIND_INTERNAL_URL = "http://${if (lib.hasInfix ":" cfg.internalListen.addr) then "[${cfg.internalListen.addr}]" else cfg.internalListen.addr}:${toString cfg.internalListen.port}";
          BSZET_MIND_API_TOKEN_FILE = "%d/api_token";
        };

        serviceConfig = {
          ExecStart = "${cfg.package}/bin/bszet-mind";
          DynamicUser = true;
          User = "bszet-mind";
          LoadCredential = [
            "username:${cfg.usernameFile}"
            "password:${cfg.passwordFile}"
            "telegram_token:${cfg.telegram.tokenFile}"
            "api_token:${cfg.apiTokenFile}"
          ];
        };
      };
    };
  };
}
