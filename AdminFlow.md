Here's the Admin User Flow Spec for DanceConnect:
                                                                                                                                                                                                       
  ---
  👨‍💼 Admin User Journey — Login → Full Control                                                                                                                                                                                                                                                                                                                                                            
   ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  Phase 1: Authentication (Same as Dancer)                                                                                                                                                             
                                                                                                                                                                                                     
  Admins use the same login as regular users, but with role: "admin" in their user object.

  Login redirects admin to: /admin/events (or dashboard with admin sidebar)

  ---
  Phase 2: Admin Dashboard Overview

  When: Admin logs in
  Goal: Central hub for platform management

  Main Admin Page (Future: /admin or /admin/dashboard)

  ┌──────────────────────────────────────────────────┐
  │  DANCECONNECT ADMIN CONTROL PANEL                │
  │                                                  │
  │  [≡] ADMIN MENU          👤 Admin User  [⋮]     │
  ├──────────────────────────────────────────────────┤
  │                                                  │
  │  QUICK STATS (Real-time)                        │
  │  ┌──────────┬──────────┬──────────┬──────────┐  │
  │  │ 1,247    │ 342      │ 89       │ 12       │  │
  │  │ USERS    │ EVENTS   │ CITIES   │ REPORTS  │  │
  │  └──────────┴──────────┴──────────┴──────────┘  │
  │                                                  │
  │  SYSTEM HEALTH                                  │
  │  Database: ✓ Healthy                            │
  │  API: ✓ Responding                              │
  │  File Storage: ✓ 2.4GB / 10GB used             │
  │                                                  │
  │  RECENT ACTIVITY                                │
  │  - 47 new user registrations (today)            │
  │  - 12 events created (today)                    │
  │  - 3 bug reports pending                        │
  │                                                  │
  └──────────────────────────────────────────────────┘

  Left Sidebar Navigation:
  ADMIN PANEL
  ─────────────────
  📊 Dashboard
  📅 Events
  👥 Users
  ⚙️  Settings
  🐛 Bug Reports
  💡 Feature Requests
  📈 Analytics
  🔐 Permissions

  ---
  Phase 3: Event Management (/admin/events)

  What Admins See:

  Events Table

  ┌────────────────────────────────────────────────────────────┐
  │ MANAGE ALL EVENTS                          [+ NEW EVENT]   │
  ├────────────────────────────────────────────────────────────┤
  │                                                            │
  │ Filter: [All Events ▼] [All Cities ▼] [Past/Future ▼]   │
  │ Search: [________________]  [⚙ ADVANCED FILTERS]         │
  │                                                            │
  │ ┌────────────────────────────────────────────────────────┐│
  │ │ EVENT TITLE        │ CITY  │ DATE      │ STATUS │ ACTS ││
  │ ├────────────────────────────────────────────────────────┤│
  │ │ Salsa Night        │ SF    │ Mar 1     │ LIVE   │ ✎ ⋮ ││
  │ │ Bachata Workshop   │ LA    │ Mar 5     │ DRAFT  │ ✎ ⋮ ││
  │ │ House Dance Battle │ NYC   │ Feb 28    │ ENDED  │ ✎ ⋮ ││
  │ │ Tango Showcase     │ SF    │ Mar 10    │ LIVE   │ ✎ ⋮ ││
  │ └────────────────────────────────────────────────────────┘│
  └────────────────────────────────────────────────────────────┘

  Row Actions (⋮ menu):
  - View Details
  - Edit Event
  - View Registrations (+ download CSV)
  - Send Announcement
  - Flag/Suspend Event
  - Delete Event

  Event Detail Modal (Edit)

  ┌──────────────────────────────────────────────┐
  │ EDIT EVENT: Salsa Night                      │
  ├──────────────────────────────────────────────┤
  │                                              │
  │ TITLE: [Salsa Night____________]            │
  │ DESCRIPTION: [________________]              │
  │ STATUS: [DRAFT ▼] → LIVE / SUSPENDED        │
  │                                              │
  │ DANCE STYLES: [✓ SALSA] [✓ SOCIAL]         │
  │ CITY: [San Francisco__]                     │
  │ STATE: [CA__]                               │
  │ DATE/TIME: [Mar 1, 2026  7:00 PM]          │
  │                                              │
  │ ORGANIZER: [DJ Alex____________]            │
  │ MAX ATTENDEES: [200__]                      │
  │ CURRENT REGISTERED: 47 / 200                │
  │                                              │
  │ TICKET PRICE: [$15.00]  [FREE ▼]           │
  │                                              │
  │ VISIBILITY: [PUBLIC ▼] (PRIVATE / HIDDEN)  │
  │                                              │
  │ [ SAVE ]  [ CANCEL ]  [ DELETE ]           │
  │                                              │
  └──────────────────────────────────────────────┘

  Registrations Sub-View

  ┌────────────────────────────────────────────────────┐
  │ EVENT REGISTRATIONS: Salsa Night (47 people)      │
  ├────────────────────────────────────────────────────┤
  │                                                   │
  │ Filter: [All ▼] [Confirmed ▼] [Waitlist ▼]     │
  │ Search: [________________]                       │
  │                                                   │
  │ ┌──────────────────────────────────────────────┐ │
  │ │ NAME          │ EMAIL        │ STATUS │ ACTS │ │
  │ ├──────────────────────────────────────────────┤ │
  │ │ Alex Rivera   │ alex@ex.com  │ ✓     │ ⋮   │ │
  │ │ Maria Santos  │ maria@ex.com │ ✓     │ ⋮   │ │
  │ │ James Chen    │ james@ex.com │ WAIT  │ ⋮   │ │
  │ └──────────────────────────────────────────────┘ │
  │                                                   │
  │ [ SEND EMAIL TO ALL ]  [ EXPORT CSV ]           │
  └────────────────────────────────────────────────────┘

  Admin Event Actions:

  ┌────────────────────┬──────────────────────────────────────────────────────────┐
  │       Action       │                       What Happens                       │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Create Event       │ Fill form → Event goes to DRAFT → Can publish when ready │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Edit Event         │ Modify any field, can push updates to attendees          │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ View Registrations │ See all registered users, export list, send emails       │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Send Announcement  │ Broadcast message to all registered dancers              │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Suspend Event      │ Event hidden from public, but registrations preserved    │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Delete Event       │ Permanently remove (with confirmation warning)           │
  ├────────────────────┼──────────────────────────────────────────────────────────┤
  │ Feature Event      │ Pin to top of dashboard for all users to see             │
  └────────────────────┴──────────────────────────────────────────────────────────┘

  ---
  Phase 4: User Management (/admin/users)

  What Admins See:

  Users Table

  ┌──────────────────────────────────────────────────────┐
  │ MANAGE USERS (1,247 total)                          │
  ├──────────────────────────────────────────────────────┤
  │                                                     │
  │ Filter: [All Roles ▼] [All User Types ▼]          │
  │ Search: [________________] [@email / name / id]    │
  │                                                     │
  │ ┌────────────────────────────────────────────────┐ │
  │ │ NAME      │ EMAIL        │ ROLE  │ TYPE │ JOIN  │ │
  │ ├────────────────────────────────────────────────┤ │
  │ │ Alex      │ alex@ex.com  │ USER  │ D    │ 2/19 │ │
  │ │ Maria     │ maria@ex.com │ USER  │ C    │ 2/15 │ │
  │ │ Admin Bob │ bob@ex.com   │ ADMIN │ -    │ 1/1  │ │
  │ │ James     │ james@ex.com │ USER  │ B    │ 2/18 │ │
  │ └────────────────────────────────────────────────┘ │
  │                                                     │
  │ Legend: D=Dancer, C=Creator, B=Both, -=Admin       │
  └──────────────────────────────────────────────────────┘

  Legend:
  - D = Dancer (attends events)
  - C = Creator (organizes events)
  - B = Both (can do both)
  - - = Admin (platform control)

  User Detail Modal

  ┌────────────────────────────────────────────┐
  │ USER PROFILE: Alex Rivera                  │
  ├────────────────────────────────────────────┤
  │                                            │
  │ EMAIL: alex@example.com                   │
  │ NAME: Alex Rivera                         │
  │ JOINED: Feb 19, 2026                      │
  │ LAST ACTIVE: Today 2:15 PM                │
  │                                            │
  │ ROLE: [USER ▼] → ADMIN / USER            │
  │ USER TYPE: [DANCER ▼] → CREATOR / BOTH   │
  │                                            │
  │ ACTIVITY                                  │
  │ - Events Registered: 12                   │
  │ - Events Created: 0                       │
  │ - Reports: 0                              │
  │                                            │
  │ DANCE STYLES: Salsa, Bachata             │
  │ LOCATION: San Francisco, CA               │
  │                                            │
  │ STATUS: [ACTIVE ▼] → SUSPENDED / BANNED  │
  │                                            │
  │ [ SAVE ]  [ SEND MESSAGE ]  [ DELETE ]   │
  │                                            │
  └────────────────────────────────────────────┘

  Admin User Actions:

  ┌───────────────────┬─────────────────────────────┬───────────────────────┐
  │      Action       │         Permission          │       When Used       │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Change Role       │ ADMIN → USER                │ Revoke admin access   │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Change User Type  │ DANCER → CREATOR / BOTH     │ User upgrades request │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Suspend User      │ Disable account temporarily │ Investigate reports   │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Ban User          │ Permanently disable         │ Abuse/violations      │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Send Message      │ Direct message              │ Warnings, updates     │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ View Activity Log │ See login history, actions  │ Audit trail           │
  ├───────────────────┼─────────────────────────────┼───────────────────────┤
  │ Delete User       │ Permanent removal           │ GDPR requests         │
  └───────────────────┴─────────────────────────────┴───────────────────────┘

  ---
  Phase 5: Bug Reports & Feature Requests

  Bug Reports (/admin/dashboard → Pending Reports section)

  ┌────────────────────────────────────────────────────┐
  │ BUG REPORTS (3 pending)                            │
  ├────────────────────────────────────────────────────┤
  │                                                   │
  │ [CRITICAL] [HIGH] [MEDIUM] [LOW]                │
  │                                                   │
  │ ┌────────────────────────────────────────────────┐│
  │ │ DATE     │ USER        │ ISSUE         │ ACTS ││
  │ ├────────────────────────────────────────────────┤│
  │ │ 2/19     │ James Chen  │ Map not load  │ ⋮   ││
  │ │ 2/19     │ Maria S.    │ Can't logout  │ ⋮   ││
  │ │ 2/18     │ Alex R.     │ Email spam    │ ⋮   ││
  │ └────────────────────────────────────────────────┘│
  │                                                   │
  └────────────────────────────────────────────────────┘

  Admin Bug Actions:
  - View full report + screenshot
  - Change priority
  - Assign to engineer
  - Mark as Fixed / Duplicate / Won't Fix
  - Comment / Note status

  Feature Requests

  Same table format, shows user-submitted wishes

  Admin can:
  - Mark as "Considering"
  - Add to roadmap
  - Close as "No"
  - Comment with reason

  ---
  Phase 6: Analytics & Reporting

  Analytics Dashboard (/admin/analytics)

  ┌──────────────────────────────────────────────────┐
  │ PLATFORM ANALYTICS                               │
  ├──────────────────────────────────────────────────┤
  │                                                 │
  │ DATE RANGE: [Last 30 days ▼]                   │
  │                                                 │
  │ GROWTH METRICS                                 │
  │ New Users (This Month): 47 ↑ 12%              │
  │ New Events (This Month): 23 ↑ 8%              │
  │ Event Registrations: 1,203 ↑ 15%              │
  │                                                 │
  │ ENGAGEMENT                                     │
  │ Avg Events/User: 3.2                          │
  │ Most Popular Style: Salsa (34%)               │
  │ Most Active City: San Francisco (28%)         │
  │                                                 │
  │ REVENUE (if ticketed)                         │
  │ Total Ticket Sales: $12,450                   │
  │ Avg Ticket Price: $12.50                      │
  │ Top Revenue Event: [House Battle: $3,200]     │
  │                                                 │
  │ CHARTS: [Line Graph] [Bar Chart] [Pie Chart]  │
  │ [ EXPORT REPORT ]  [ EMAIL REPORT ]           │
  │                                                 │
  └──────────────────────────────────────────────────┘

  ---
  Phase 7: Settings & Configuration

  Admin Settings (/admin/settings)

  ┌────────────────────────────────────────────────┐
  │ ADMIN SETTINGS                                 │
  ├────────────────────────────────────────────────┤
  │                                                │
  │ PLATFORM CONFIGURATION                        │
  │ ┌──────────────────────────────────────────┐  │
  │ │ Site Name: DANCECONNECT                  │  │
  │ │ Support Email: support@danceconnect.com  │  │
  │ │ Help URL: help.danceconnect.com          │  │
  │ │ Maintenance Mode: [ OFF ] ← Turn ON      │  │
  │ └──────────────────────────────────────────┘  │
  │                                                │
  │ PERMISSIONS & ACCESS                          │
  │ ┌──────────────────────────────────────────┐  │
  │ │ ☑ Allow user signups                    │  │
  │ │ ☑ Allow event creation                  │  │
  │ │ ☐ Require email verification            │  │
  │ │ ☐ Limit events per user (5/month)      │  │
  │ └──────────────────────────────────────────┘  │
  │                                                │
  │ NOTIFICATIONS                                 │
  │ ☑ Email me on new bug reports                 │
  │ ☑ Email me on new feature requests            │
  │ ☑ Email me on user flagged events            │
  │ ☑ Weekly stats digest                         │
  │                                                │
  │ [ SAVE ]  [ RESET TO DEFAULT ]               │
  │                                                │
  └────────────────────────────────────────────────┘

  ---
  Phase 8: Permissions & Admin Hierarchy

  Admin Roles (Future Enhancement):

  ┌─────────────────────┬─────────────┬─────────────┬─────────┐
  │     Permission      │ Super Admin │ Event Admin │ Support │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Manage Users        │ ✓           │ ✗           │ ✗       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Manage Events       │ ✓           │ ✓           │ ✗       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ View Analytics      │ ✓           │ ✓           │ ✓       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Respond to Reports  │ ✓           │ ✗           │ ✓       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Manage Other Admins │ ✓           │ ✗           │ ✗       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Change Settings     │ ✓           │ ✗           │ ✗       │
  ├─────────────────────┼─────────────┼─────────────┼─────────┤
  │ Ban Users           │ ✓           │ ✗           │ ✗       │
  └─────────────────────┴─────────────┴─────────────┴─────────┘

  ---
  Phase 9: Admin Workflows

  Workflow 1: Handle Bug Report

  1. Admin sees notification: "New bug report"
  2. Click → View bug detail (screenshot, description, user)
  3. Verify the issue (might test locally)
  4. Assign priority (Critical/High/Medium/Low)
  5. Comment: "Looking into this"
  6. Mark as "In Progress"
  7. After fix deployed: Mark as "Fixed"
  8. Notify user: "Bug fixed, thank you for reporting!"

  Workflow 2: Suspend Inappropriate User

  1. Receive report: User posted offensive event
  2. Go to User Management
  3. Find user in table
  4. Click → View profile
  5. Change STATUS: ACTIVE → SUSPENDED
  6. Add note: "Violated community guidelines"
  7. Auto-email user: "Your account is suspended. Appeal here: ..."
  8. Event is hidden from platform
  9. User can't create new events until unsuspended

  Workflow 3: Promote Creator to Admin

  1. Go to User Management
  2. Search for user "Maria Santos"
  3. Click → Edit
  4. Change ROLE: USER → ADMIN
  5. Change USER TYPE: CREATOR → -
  6. Save
  7. Maria receives email: "You're now an admin!"
  8. Maria can now access /admin/* pages

  Workflow 4: Monitor Event Registrations

  1. Go to Events
  2. Click event "Salsa Night"
  3. Click "View Registrations"
  4. See 47 people registered
  5. Notice one user flagged as "suspicious"
  6. Click user → View profile → Check activity
  7. Decide: Accept / Kick from event
  8. Send announcement: "Event is at capacity, waitlist opens here"

  ---
  Phase 10: Admin Data Model

  // Admin user object
  {
    id: "uuid",
    email: "admin@danceconnect.com",
    full_name: "Admin Bob",
    role: "admin",  // ONLY role that grants /admin/* access
    user_type: null,  // Admins don't have user_type

    // Admin-specific
    permissions: {
      manage_users: true,
      manage_events: true,
      view_analytics: true,
      respond_reports: true,
      manage_admins: true,
      change_settings: true
    },

    created_at: "2026-01-01T00:00:00Z",
    last_login: "2026-02-19T14:30:00Z"
  }

  ---
  Phase 11: Core Admin Features Summary

  ┌──────────────────┬──────────────────┬─────────────────────────────────────────────┐
  │     Feature      │       Page       │                   Purpose                   │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Event Management │ /admin/events    │ Create, edit, delete, suspend events        │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ User Management  │ /admin/users     │ Manage roles, suspend/ban, view activity    │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Bug Reports      │ /admin/dashboard │ Triage and track bug fixes                  │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Feature Requests │ /admin/dashboard │ See user suggestions, prioritize            │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Analytics        │ /admin/analytics │ Growth metrics, engagement, revenue         │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Settings         │ /admin/settings  │ Platform config, permissions, notifications │
  ├──────────────────┼──────────────────┼─────────────────────────────────────────────┤
  │ Dashboard        │ /admin           │ Quick stats, health, recent activity        │
  └──────────────────┴──────────────────┴─────────────────────────────────────────────┘

  ---
  Phase 12: Admin Permissions & Access Control

  Access Rules:
  IF user.role !== "admin"
    THEN block access to /admin/*

  IF user in /admin/users
    AND user.role === "admin"
    THEN show full user list with edit controls

  IF user creates event
    AND user.user_type === "creator"
    THEN event auto-goes to DRAFT (requires admin review? → configurable)

  ---
  Phase 13: Success Metrics for Admin Interface

  An admin should be able to:
  - ✓ See all events on the platform at a glance
  - ✓ Manage event visibility & details in <1 min
  - ✓ Find and manage any user in <30 seconds
  - ✓ Respond to bug reports without leaving the platform
  - ✓ View platform growth metrics in real-time
  - ✓ Handle moderation tasks (suspend/ban) quickly
  - ✓ Understand what users are doing (analytics)

  ---
  Summary: What Admins See & Do

  LOGIN → ADMIN DASHBOARD → [CHOOSE PATH]
                            ├→ Events → Edit/Delete/Suspend/Feature
                            ├→ Users → Change Role/Ban/Monitor
                            ├→ Reports → Triage/Assign/Track
                            ├→ Analytics → View Growth/Engagement
                            └→ Settings → Configure Platform

  ---
  