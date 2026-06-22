/**
 * v0.23.20: DB 连接池状态轮询 Hook
 *
 * 每 5 秒轮询后端 get_db_pool_status 命令，返回连接池使用情况。
 * 用于 FrontstageHeader 状态栏实时显示 DB 连接池耗尽预警。
 */

import { useQuery } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/api/core';

export interface DbPoolStatus {
  max_size: number;
  connections: number;
  idle: number;
  in_use: number;
  connection_timeout_secs: number;
}

export function useDbPoolStatus() {
  return useQuery<DbPoolStatus>({
    queryKey: ['db-pool-status'],
    queryFn: async () => {
      return await loggedInvoke<DbPoolStatus>('get_db_pool_status');
    },
    refetchInterval: 5000,
    staleTime: 3000,
  });
}
