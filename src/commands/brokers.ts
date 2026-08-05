import { getRunEnv, RUN_ENV, invokeTauri, invokeWeb, logger } from "@/adapters";

export interface BrokerConnectionDto {
  id: string;
  accountId: string;
  provider: string;
  createdAt: string;
  updatedAt: string;
}

export const getAuthUrl = async (provider: string, profileId?: string): Promise<string> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("get_auth_url", { provider, profileId });
      case RUN_ENV.WEB:
        return invokeWeb("get_auth_url", { provider, profileId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error getting auth URL.");
    throw error;
  }
};

export const handleOauthCallback = async (accountId: string, provider: string, profileId?: string): Promise<BrokerConnectionDto> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("handle_oauth_callback", { accountId, provider, profileId });
      case RUN_ENV.WEB:
        return invokeWeb("handle_oauth_callback", { accountId, provider, profileId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error handling OAuth callback.");
    throw error;
  }
};

export const getBrokerConnection = async (accountId: string, profileId?: string): Promise<BrokerConnectionDto | null> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("get_broker_connection", { accountId, profileId });
      case RUN_ENV.WEB:
        return invokeWeb("get_broker_connection", { accountId, profileId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error getting broker connection.");
    throw error;
  }
};

export const fetchBrokerActivities = async (accountId: string, profileId?: string): Promise<any[]> => {
  try {
    switch (getRunEnv()) {
      case RUN_ENV.DESKTOP:
        return invokeTauri("fetch_broker_activities", { accountId, profileId });
      case RUN_ENV.WEB:
        return invokeWeb("fetch_broker_activities", { accountId, profileId });
      default:
        throw new Error(`Unsupported`);
    }
  } catch (error) {
    logger.error("Error fetching broker activities.");
    throw error;
  }
};
