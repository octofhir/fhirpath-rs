import { Component, createSignal, Show } from 'solid-js';
import styles from './FileUpload.module.css';

interface FileUploadProps {
  onFileUploaded: (filename: string) => void;
}

const FileUpload: Component<FileUploadProps> = (props) => {
  const [isDragging, setIsDragging] = createSignal(false);
  const [isUploading, setIsUploading] = createSignal(false);
  const [uploadStatus, setUploadStatus] = createSignal<{
    type: 'success' | 'error' | null;
    message: string;
  }>({ type: null, message: '' });

  let fileInputRef: HTMLInputElement | undefined;

  const validateFile = (file: File): string | null => {
    // Check file type
    if (file.type !== 'application/json' && !file.name.endsWith('.json')) {
      return 'Please upload a JSON file';
    }

    // Check file size (max 10MB)
    if (file.size > 60 * 1024 * 1024) {
      return 'File size must be less than 10MB';
    }

    return null;
  };

  const uploadFile = async (file: File) => {
    const validationError = validateFile(file);
    if (validationError) {
      setUploadStatus({ type: 'error', message: validationError });
      return;
    }

    setIsUploading(true);
    setUploadStatus({ type: null, message: '' });

    try {
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch('/files/upload', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Upload failed' }));
        throw new Error(errorData.error || 'Upload failed');
      }

      const result = await response.json();
      setUploadStatus({ 
        type: 'success', 
        message: `Successfully uploaded ${result.filename}` 
      });
      props.onFileUploaded(result.filename);
    } catch (error) {
      setUploadStatus({ 
        type: 'error', 
        message: error instanceof Error ? error.message : 'Upload failed' 
      });
    } finally {
      setIsUploading(false);
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      uploadFile(files[0]);
    }
  };

  const handleFileSelect = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (files && files.length > 0) {
      uploadFile(files[0]);
    }
    // Reset the input value so the same file can be selected again
    target.value = '';
  };

  const handleClickUpload = () => {
    fileInputRef?.click();
  };

  return (
    <div class={styles.fileUpload}>
      <div
        class={`${styles.dropZone} ${isDragging() ? styles.dropZoneActive : ''}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={handleClickUpload}
      >
        <div class={styles.dropZoneIcon}>📁</div>
        <div class={styles.dropZoneText}>
          {isDragging() ? 'Drop your file here' : 'Drag & drop a JSON file here'}
        </div>
        <div class={styles.dropZoneSubtext}>
          or click to browse files (JSON only, max 60MB)
        </div>
        
        <button 
          class={`button button-secondary ${styles.uploadButton}`}
          disabled={isUploading()}
          onClick={(e) => {
            e.stopPropagation();
            handleClickUpload();
          }}
        >
          📎 Choose File
        </button>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        class={styles.hiddenInput}
        onChange={handleFileSelect}
      />

      <Show when={isUploading()}>
        <div class={`${styles.uploadStatus} ${styles.uploadLoading}`}>
          <div class={styles.spinner}></div>
          Uploading file...
        </div>
      </Show>

      <Show when={uploadStatus().type}>
        <div 
          class={`${styles.uploadStatus} ${
            uploadStatus().type === 'success' 
              ? styles.uploadSuccess 
              : styles.uploadError
          }`}
        >
          {uploadStatus().message}
        </div>
      </Show>
    </div>
  );
};

export default FileUpload;