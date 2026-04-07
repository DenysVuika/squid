import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';
import userEvent from '@testing-library/user-event';
import { NavUser } from './nav-user';
import { SidebarProvider } from '@/components/ui/sidebar';
import { MemoryRouter } from 'react-router-dom';

// ─── JSDOM polyfills required by Radix UI ────────────────────────────────────

beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

// ─── Mocks ───────────────────────────────────────────────────────────────────

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router-dom')>();
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// ─── Fixtures ────────────────────────────────────────────────────────────────

const DEFAULT_USER = {
  name: 'Test User',
  email: 'test@example.com',
  avatar: 'https://example.com/avatar.png',
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

const renderWithProviders = (user = DEFAULT_USER) => {
  return render(
    <MemoryRouter>
      <SidebarProvider defaultOpen={true}>
        <NavUser user={user} />
      </SidebarProvider>
    </MemoryRouter>
  );
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('NavUser', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Basic rendering ────────────────────────────────────────────────────────

  it('renders user name in the trigger button', () => {
    renderWithProviders(DEFAULT_USER);
    expect(screen.getByText('Test User')).toBeInTheDocument();
  });

  it('renders user email in the trigger button', () => {
    renderWithProviders(DEFAULT_USER);
    expect(screen.getByText('test@example.com')).toBeInTheDocument();
  });

  it('renders the avatar fallback with initials', () => {
    renderWithProviders(DEFAULT_USER);
    expect(screen.getByText('CN')).toBeInTheDocument();
  });

  it('renders the ChevronsUpDown icon', () => {
    const { container } = renderWithProviders(DEFAULT_USER);
    const svgs = container.querySelectorAll('svg');
    expect(svgs.length).toBeGreaterThan(0);
  });

  // ── Dropdown menu content ──────────────────────────────────────────────────

  it('shows user name in dropdown label when opened', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const nameElements = screen.getAllByText('Test User');
    expect(nameElements.length).toBeGreaterThanOrEqual(2);
  });

  it('shows avatar fallback in dropdown content when opened', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const fallbacks = screen.getAllByText('CN');
    expect(fallbacks.length).toBeGreaterThanOrEqual(2);
  });

  // ── Navigation items ───────────────────────────────────────────────────────

  it('"Logs" menu item navigates to /logs when clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const logsItem = await screen.findByText('Logs');
    await user.click(logsItem);

    expect(mockNavigate).toHaveBeenCalledWith('/logs');
  });

  it('"Agent Statistics" menu item navigates to /agent-stats when clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const statsItem = await screen.findByText('Agent Statistics');
    await user.click(statsItem);

    expect(mockNavigate).toHaveBeenCalledWith('/agent-stats');
  });

  // ── Disabled items ─────────────────────────────────────────────────────────

  it('has "Upgrade to Pro" item disabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const upgradeItem = await screen.findByText('Upgrade to Pro');
    expect(upgradeItem.closest('[data-disabled]')).toHaveAttribute('data-disabled');
  });

  it('has "Account" item disabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const accountItem = await screen.findByText('Account');
    expect(accountItem.closest('[data-disabled]')).toHaveAttribute('data-disabled');
  });

  it('has "Billing" item disabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const billingItem = await screen.findByText('Billing');
    expect(billingItem.closest('[data-disabled]')).toHaveAttribute('data-disabled');
  });

  it('has "Notifications" item disabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const notificationsItem = await screen.findByText('Notifications');
    expect(notificationsItem.closest('[data-disabled]')).toHaveAttribute('data-disabled');
  });

  it('has "Log out" item disabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    const logoutItem = await screen.findByText('Log out');
    expect(logoutItem.closest('[data-disabled]')).toHaveAttribute('data-disabled');
  });

  // ── Responsive behavior ────────────────────────────────────────────────────

  it('renders dropdown menu when opened on mobile', async () => {
    const user = userEvent.setup();
    window.innerWidth = 768;
    window.dispatchEvent(new Event('resize'));

    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    expect(await screen.findByRole('menu')).toBeInTheDocument();
  });

  it('renders dropdown menu when opened on desktop', async () => {
    const user = userEvent.setup();
    window.innerWidth = 1024;
    window.dispatchEvent(new Event('resize'));

    renderWithProviders(DEFAULT_USER);

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    expect(await screen.findByRole('menu')).toBeInTheDocument();
  });
});
