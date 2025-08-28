import { Component, createSignal, createEffect, onMount, Show, For } from 'solid-js';
import styles from './FileList.module.css';
import { FileInfo } from '../services/types';

interface FileListProps {
  selectedFile: string;
  onFileSelect: (filename: string) => void;
  refreshTrigger?: number;
}

const FileList: Component<FileListProps> = (props) => {
  const [files, setFiles] = createSignal<FileInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal('');

  const loadFiles = async () => {
    setLoading(true);
    setError('');
    
    try {
      const response = await fetch('/files');
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Failed to load files' }));
        throw new Error(errorData.error || 'Failed to load files');
      }
      
      const data = await response.json();
      setFiles(data.files || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load files');
    } finally {
      setLoading(false);
    }
  };

  const deleteFile = async (filename: string) => {
    if (!confirm(`Are you sure you want to delete "${filename}"?`)) {
      return;
    }

    try {
      const response = await fetch(`/files/${encodeURIComponent(filename)}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Failed to delete file' }));
        throw new Error(errorData.error || 'Failed to delete file');
      }

      // If the deleted file was selected, clear selection
      if (props.selectedFile === filename) {
        props.onFileSelect('');
      }

      // Reload the file list
      await loadFiles();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete file');
    }
  };

  const previewFile = async (filename: string) => {
    try {
      const response = await fetch(`/files/${encodeURIComponent(filename)}`);
      
      if (!response.ok) {
        throw new Error('Failed to load file');
      }
      
      const fileContent = await response.json();
      
      // Open preview in a new window/tab
      const previewWindow = window.open('', '_blank');
      if (previewWindow) {
        previewWindow.document.write(`
          <html>
            <head>
              <title>Preview: ${filename}</title>
              <style>
                body { 
                  font-family: monospace; 
                  margin: 20px; 
                  background: #f8f9fa;
                }
                pre { 
                  background: white;
                  padding: 20px; 
                  border-radius: 4px;
                  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                  overflow: auto;
                }
              </style>
            </head>
            <body>
              <h2>File: ${filename}</h2>
              <pre>${JSON.stringify(fileContent, null, 2)}</pre>
            </body>
          </html>
        `);
        previewWindow.document.close();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to preview file');
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const formatDate = (dateString: string): string => {
    try {
      console.log(dateString);
      return new Date(dateString).toLocaleString()
    } catch {
      return dateString;
    }
  };

  onMount(() => {
    loadFiles();
  });

  // React to refresh trigger
  createEffect(() => {
    if (props.refreshTrigger !== undefined) {
      loadFiles();
    }
  });

  const handleFileSelect = (filename: string) => {
    if (props.selectedFile === filename) {
      props.onFileSelect(''); // Deselect if already selected
    } else {
      props.onFileSelect(filename);
    }
  };

  return (
    <div class={styles.fileList}>
      <div class={styles.header}>
        <div class={styles.title}>Stored Files</div>
        <button
          class={styles.refreshButton}
          onClick={loadFiles}
          disabled={loading()}
        >
          🔄 Refresh
        </button>
      </div>

      <Show when={props.selectedFile}>
        <div class={styles.selectedIndicator}>
          ✓ Selected: {props.selectedFile}
        </div>
      </Show>

      <Show when={loading()}>
        <div class={styles.loading}>
          <div class={styles.spinner}></div>
          <span>Loading files...</span>
        </div>
      </Show>

      <Show when={error() && !loading()}>
        <div class={styles.error}>
          ❌ {error()}
        </div>
      </Show>

      <Show when={!loading() && !error() && files().length === 0}>
        <div class={`${styles.list} ${styles.emptyState}`}>
          <div class={styles.emptyIcon}>📂</div>
          <div class={styles.emptyText}>No files found</div>
          <div class={styles.emptySubtext}>
            Upload a JSON file to get started
          </div>
        </div>
      </Show>

      <Show when={!loading() && !error() && files().length > 0}>
        <div class={styles.list}>
          <For each={files()}>
            {(file) => (
              <div
                class={`${styles.fileItem} ${
                  props.selectedFile === file.name ? styles.fileItemSelected : ''
                }`}
                onClick={() => handleFileSelect(file.name)}
              >
                <div class={styles.fileInfo}>
                  <div class={styles.fileName}>
                    <span class={styles.fileIcon}>📄</span>
                    <span class={styles.fileNameText} title={file.name}>
                      {file.name}
                    </span>
                  </div>
                  <div class={styles.fileMeta}>
                    <div class={styles.fileMetaItem}>
                      <span>📏</span>
                      <span>{formatFileSize(file.size)}</span>
                    </div>
                    <div class={styles.fileMetaItem}>
                      <span>📅</span>
                      <span>{formatDate(file.modified)}</span>
                    </div>
                    <Show when={file.type}>
                      <div class={styles.fileMetaItem}>
                        <span>🏷️</span>
                        <span>{file.type}</span>
                      </div>
                    </Show>
                  </div>
                </div>

                <div class={styles.fileActions}>
                  <button
                    class={`${styles.actionButton} ${styles.previewButton}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      previewFile(file.name);
                    }}
                    title="Preview file"
                  >
                    👁️
                  </button>
                  <button
                    class={`${styles.actionButton} ${styles.deleteButton}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteFile(file.name);
                    }}
                    title="Delete file"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default FileList;