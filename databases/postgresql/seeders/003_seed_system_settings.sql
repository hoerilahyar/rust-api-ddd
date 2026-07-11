INSERT INTO system_settings (key, value, description) VALUES
    ('storage_provider',        'local', 'Default media storage provider: local or s3'),
    ('max_upload_size_mb',      '20',    'Maximum upload size for product media, in MB'),
    ('pagination_default_limit','20',    'Default page size for list endpoints')
ON CONFLICT (key) DO NOTHING;
