import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { supportApi } from '../../services/api';

export default function BugReport() {
  const navigate = useNavigate();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [screenshot, setScreenshot] = useState(null);
  const [submitting, setSubmitting] = useState(false);
  const [showToast, setShowToast] = useState(false);

  function handleFileSelect(e) {
    const file = e.target.files?.[0];
    if (file) setScreenshot(URL.createObjectURL(file));
  }

  async function handleSubmit(e) {
    e.preventDefault();
    setSubmitting(true);
    try {
      await supportApi.submitBugReport(title, description, null);
      setShowToast(true);
      setTimeout(() => navigate('/settings'), 1200);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-6)' }}>Report a Bug</h1>

      <form onSubmit={handleSubmit}>
        <label className="ck-bugreport__upload">
          {screenshot ? (
            <img src={screenshot} alt="Screenshot preview" className="ck-bugreport__preview" />
          ) : (
            <span className="text-secondary">Tap to add a screenshot</span>
          )}
          <input type="file" accept="image/*" onChange={handleFileSelect} style={{ display: 'none' }} />
        </label>

        <Input id="bug-title" placeholder="Title" value={title} onChange={(e) => setTitle(e.target.value)} />

        <div className="ck-input-group">
          <textarea
            className="ck-input"
            style={{ width: '100%', minHeight: 120, background: 'var(--bg-surface)', border: '1.5px solid var(--border-subtle)', borderRadius: 'var(--radius-button)', padding: 'var(--space-3)' }}
            placeholder="Describe the issue..."
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        <Button type="submit" loading={submitting} loadingLabel="Submitting...">Submit</Button>
      </form>

      <Toast visible={showToast} message="Your report has been submitted." onDismiss={() => setShowToast(false)} />
    </div>
  );
}
