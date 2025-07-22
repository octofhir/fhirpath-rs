/**
 * Loading spinner component for comparison page
 */

import React from 'react';
import { Loader, Stack, Text } from '@mantine/core';
import styles from './common.module.css';

interface LoadingSpinnerProps {
  message?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
}

export function LoadingSpinner({
  message = 'Loading comparison results...',
  size = 'lg',
  className
}: LoadingSpinnerProps) {
  return (
    <div className={`${styles.loadingContainer} ${className || ''}`}>
      <Stack align="center" gap="md">
        <Loader size={size} color="blue" />
        <Text size="sm" c="dimmed" ta="center">
          {message}
        </Text>
      </Stack>
    </div>
  );
}
