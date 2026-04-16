import { Component } from 'react';
import { Button, Result } from 'antd';

/**
 * Catches render errors in the subtree and shows a fallback UI.
 * Wrap around any feature slice that should degrade gracefully.
 */
export class ErrorBoundary extends Component {
  state = { error: null };

  static getDerivedStateFromError(error) {
    return { error };
  }

  componentDidCatch(error, info) {
    console.error('[ErrorBoundary]', error, info);
  }

  reset = () => this.setState({ error: null });

  render() {
    if (this.state.error) {
      return (
        <Result
          status="error"
          title="Something went wrong"
          subTitle={this.state.error?.message}
          extra={
            <Button type="primary" onClick={this.reset}>
              Try again
            </Button>
          }
        />
      );
    }
    return this.props.children;
  }
}
