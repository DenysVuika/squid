import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import { Button } from '../ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Badge } from '../ui/badge';
import { Spinner } from '../ui/spinner';

interface LogEntry {
  id: number;
  timestamp: number;
  level: string;
  target: string;
  message: string;
  session_id: string | null;
}

interface LogsResponse {
  logs: LogEntry[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export default function Logs() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  const [levelFilter, setLevelFilter] = useState<string>('all');
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    fetchLogs();
  }, [page, pageSize, levelFilter]);

  const fetchLogs = async () => {
    setLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams({
        page: page.toString(),
        page_size: pageSize.toString(),
      });

      if (levelFilter !== 'all') {
        params.append('level', levelFilter);
      }

      const response = await fetch(`/api/logs?${params}`);

      if (!response.ok) {
        throw new Error('Failed to fetch logs');
      }

      const data: LogsResponse = await response.json();
      setLogs(data.logs);
      setTotalPages(data.total_pages);
      setTotal(data.total);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  const getLevelBadgeVariant = (level: string): 'default' | 'secondary' | 'destructive' | 'outline' => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'destructive';
      case 'warn':
        return 'outline';
      case 'info':
        return 'default';
      case 'debug':
      case 'trace':
        return 'secondary';
      default:
        return 'default';
    }
  };

  const handlePreviousPage = () => {
    if (page > 1) {
      setPage(page - 1);
    }
  };

  const handleNextPage = () => {
    if (page < totalPages) {
      setPage(page + 1);
    }
  };

  const handlePageSizeChange = (value: string) => {
    setPageSize(Number(value));
    setPage(1); // Reset to first page when changing page size
  };

  const handleLevelFilterChange = (value: string) => {
    setLevelFilter(value);
    setPage(1); // Reset to first page when changing filter
  };

  return (
    <div className="flex h-screen w-screen flex-col p-6 gap-6 bg-gray-50">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Application Logs</h1>
          <p className="text-gray-600 mt-1">
            {total > 0 ? `Showing ${total} log ${total === 1 ? 'entry' : 'entries'}` : 'No logs found'}
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Log Entries</CardTitle>
              <CardDescription>
                View and filter application logs with pagination
              </CardDescription>
            </div>
            <div className="flex gap-4 items-center">
              <div className="flex items-center gap-2">
                <span className="text-sm text-gray-600">Level:</span>
                <Select value={levelFilter} onValueChange={handleLevelFilterChange}>
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    <SelectItem value="error">Error</SelectItem>
                    <SelectItem value="warn">Warn</SelectItem>
                    <SelectItem value="info">Info</SelectItem>
                    <SelectItem value="debug">Debug</SelectItem>
                    <SelectItem value="trace">Trace</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-sm text-gray-600">Per page:</span>
                <Select value={pageSize.toString()} onValueChange={handlePageSizeChange}>
                  <SelectTrigger className="w-24">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="25">25</SelectItem>
                    <SelectItem value="50">50</SelectItem>
                    <SelectItem value="100">100</SelectItem>
                    <SelectItem value="200">200</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Spinner className="h-8 w-8" />
            </div>
          ) : error ? (
            <div className="text-center py-12">
              <p className="text-red-600">{error}</p>
              <Button onClick={fetchLogs} className="mt-4">
                Retry
              </Button>
            </div>
          ) : logs.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              No logs found with the current filters
            </div>
          ) : (
            <>
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[180px]">Timestamp</TableHead>
                      <TableHead className="w-[100px]">Level</TableHead>
                      <TableHead className="w-[200px]">Target</TableHead>
                      <TableHead>Message</TableHead>
                      <TableHead className="w-[150px]">Session ID</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {logs.map((log) => (
                      <TableRow key={log.id}>
                        <TableCell className="font-mono text-xs">
                          {formatTimestamp(log.timestamp)}
                        </TableCell>
                        <TableCell>
                          <Badge variant={getLevelBadgeVariant(log.level)}>
                            {log.level.toUpperCase()}
                          </Badge>
                        </TableCell>
                        <TableCell className="font-mono text-xs text-gray-600">
                          {log.target}
                        </TableCell>
                        <TableCell className="max-w-md truncate" title={log.message}>
                          {log.message}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-gray-500">
                          {log.session_id ? (
                            <span className="truncate block" title={log.session_id}>
                              {log.session_id.slice(0, 8)}...
                            </span>
                          ) : (
                            <span className="text-gray-400">-</span>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              <div className="flex items-center justify-between mt-4">
                <div className="text-sm text-gray-600">
                  Page {page} of {totalPages} ({total} total {total === 1 ? 'entry' : 'entries'})
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    onClick={handlePreviousPage}
                    disabled={page === 1}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    onClick={handleNextPage}
                    disabled={page >= totalPages}
                  >
                    Next
                  </Button>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
