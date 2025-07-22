/**
 * Error boundary component for comparison page
 */
import {type ReactNode, Component, useCallback, useState, useEffect} from 'react';

import { Alert, Button, Stack, Text, Title } from '@mantine/core';
import { IconAlertCircle, IconRefresh } from '@tabler/icons-react';
import styles from './common.module.css';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onReset?: () => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.setState({
      error,
      errorInfo
    });
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    });

    if (this.props.onReset) {
      this.props.onReset();
    }
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className={styles.errorContainer}>
          <Alert
            icon={<IconAlertCircle size="1rem" />}
            title="Something went wrong"
            color="red"
            variant="light"
          >
            <Stack gap="md">
              <Text size="sm">
                An error occurred while rendering the comparison page. This might be due to:
              </Text>
              <ul className={styles.errorList}>
                <li>Network connectivity issues</li>
                <li>Invalid data format</li>
                <li>Browser compatibility problems</li>
              </ul>

              {process.env.NODE_ENV === 'development' && this.state.error && (
                <details className={styles.errorDetails}>
                  <summary>Error Details (Development)</summary>
                  <pre className={styles.errorStack}>
                    {this.state.error.toString()}
                    {this.state.errorInfo?.componentStack}
                  </pre>
                </details>
              )}

              <Button
                leftSection={<IconRefresh size="1rem" />}
                onClick={this.handleReset}
                variant="light"
                color="red"
                size="sm"
              >
                Try Again
              </Button>
            </Stack>
          </Alert>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Hook-based error boundary for functional components
 */
export function useErrorHandler() {
  const [error, setError] = useState<Error | null>(null);

  const resetError = useCallback(() => {
    setError(null);
  }, []);

  const handleError = useCallback((error: Error) => {
    console.error('Error caught by useErrorHandler:', error);
    setError(error);
  }, []);

  useEffect(() => {
    if (error) {
      throw error;
    }
  }, [error]);

  return { handleError, resetError };
}
