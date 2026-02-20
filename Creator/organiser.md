Creator/Organizer User Flow Specification         
                                                   
  Overview
                                                                                                                                
  Event creators and organizers use DanceConnect to create dance events, manage attendee registrations, track attendance, and
  monitor event performance. They have elevated permissions compared to regular dancers and access to event analytics and       
  management tools.                                          
                                         
  ---
  1. Authentication & Onboarding

  Entry Point: Login / Register

  - Login flow: Email + password
    - Verify credentials against User entity
    - Set localStorage token (Bearer)
    - Redirect to /dashboard if already has organizer profile
    - Redirect to onboarding if first-time organizer
  - Register flow:
    - Email, password, full name
    - Option to "Register as Organizer" (checkbox or separate button)
    - If organizer selected → proceed to organizer onboarding
    - If dancer only → redirect to dancer dashboard

  Organizer Onboarding (/edit-profile or dedicated flow)

  Fields:
  - Organization name (optional)
  - Bio/description (250 chars)
  - Profile image (upload via UploadFile)
  - Phone number (for event inquiries)
  - Preferred dance styles (multi-select)
  - Website (optional)
  - Social media links (optional)

  Validations:
  - Email unique
  - Password 8+ chars
  - Organization name max 100 chars
  - Profile image < 5MB

  Completion:
  - Save to User entity with role: 'organizer'
  - Redirect to /dashboard
  - Show welcome banner: "Your organizer profile is live!"

  ---
  2. Main Dashboard (/dashboard)

  Hero Section

  ┌─────────────────────────────────────────┐
  │ Welcome back, [Name]!                   │
  │ You have 3 active events                │
  └─────────────────────────────────────────┘

  Key Sections

  A. Quick Stats Panel (StatsPanel modified for organizers)
  - Total Events: Count of all events created by this organizer
  - Total Registrations: Sum of current_attendees across all their events
  - Upcoming Events: Count with start_date > today
  - Conversion Rate: (total_registrations / total_capacity) × 100% (optional advanced metric)

  B. Events Grid (EventCard modified for creator view)
  - Show all events created by this organizer
  - For each event card:
    - Event image/gradient
    - Event title, date, location
    - Attendee count vs max capacity: "24/50 dancers"
    - Status badge: "Active", "Past", "Draft" (if not published)
    - Quick actions:
        - Edit (pencil icon)
      - View registrations (person icon)
      - Analytics (chart icon)
      - Delete (trash icon)
  - Filter options: "Active", "Upcoming", "Past", "All"
  - Sort options: "Date", "Recent", "Most Popular"

  C. Recent Registrations (new section)
  Recent sign-ups across all events:
  - Dancer Name → Event Name → 2 hours ago
  - Dancer Name → Event Name → 1 day ago
  - Dancer Name → Event Name → 3 days ago
  - Click to view full registrations for that event

  D. Floating Action Button (FAB)
  - Primary: "+ CREATE EVENT" → /create-event
  - Secondary menu:
    - View all registrations
    - Analytics dashboard
    - Event templates (future feature)

  ---
  3. Core Activities

  A. Create Event (/create-event)

  Form Fields:
  ┌─────────────────────────────────────────┐
  │ EVENT DETAILS                           │
  ├─────────────────────────────────────────┤
  │ Title *              [_________________] │
  │ Description *        [_________________] │
  │                      [_________________] │
  │ Date & Time *        [Date] [Time]      │
  │ End Time             [Time]             │
  │ Location *           [_________________] │
  │ Venue Name           [_________________] │
  │ Address              [_________________] │
  │ City *               [_________________] │
  │ State *              [_________________] │
  │                                         │
  │ REGISTRATION SETTINGS                   │
  ├─────────────────────────────────────────┤
  │ Max Attendees        [_________________] │
  │ Ticket Price ($)     [_________________] │
  │ Event Type *         [Dropdown]         │
  │ Dance Styles *       [Multi-select]     │
  │                                         │
  │ MEDIA                                   │
  ├─────────────────────────────────────────┤
  │ Event Image/Banner   [Upload]           │
  │                                         │
  │ [ DRAFT ]  [ PUBLISH ]                  │
  └─────────────────────────────────────────┘

  Validations:
  - Title required, max 150 chars
  - Description required, max 1000 chars
  - Date must be future date
  - Location required
  - Max attendees > 0
  - At least one dance style selected
  - Image < 5MB

  On Save:
  - If DRAFT: Save with published: false, allow edit later
  - If PUBLISH: Save with published: true, event visible to dancers
  - Redirect to event detail page

  Data Model:
  {
    id: uuid,
    organizer_id: user_id,
    title: string,
    description: string,
    start_date: ISO datetime,
    end_date: ISO datetime,
    venue_name: string,
    address: string,
    city: string,
    state: string,
    ticket_price: number,
    max_attendees: number,
    current_attendees: number,
    event_type: string,
    dance_styles: array,
    image_url: string,
    published: boolean,
    created_at: ISO datetime,
    updated_at: ISO datetime
  }

  ---
  B. Edit Event (/create-event?id=[eventId])

  Behavior:
  - Load event data from Event.get(eventId)
  - Check organizer_id === current user (403 if not organizer)
  - Show same form as Create Event
  - "DRAFT" → "SAVE" button (republish)
  - "PUBLISH" → "UPDATE" button
  - Cannot edit past events (disable form)
  - Show "Last updated: X days ago" timestamp

  Validations: Same as Create

  On Update:
  - Call Event.update(eventId, data)
  - Show toast: "Event updated successfully"
  - Stay on page or redirect to detail view

  ---
  C. View Registrations (/event?id=[eventId] + new tab or modal)

  Access:
  - Show button on event detail for organizer only
  - Or click "View registrations" from event card

  Display:
  ┌─────────────────────────────────────────┐
  │ REGISTRATIONS: [Event Title]            │
  │ 24 registered / 50 capacity (48%)       │
  ├─────────────────────────────────────────┤
  │ [Search] [Filter] [Export]              │
  ├─────────────────────────────────────────┤
  │ Dancer Name    | Email      | Join Date │
  ├─────────────────────────────────────────┤
  │ Alice Smith    | a@dance.io | 2025-02-10│
  │ Bob Johnson    | b@dance.io | 2025-02-12│
  │ Carol Davis    | c@dance.io | 2025-02-15│
  │ ... (paginate 20 per page)              │
  ├─────────────────────────────────────────┤
  │ [Export CSV]  [Send Reminder]           │
  └─────────────────────────────────────────┘

  Features:
  - Sort by: Name, Join Date, Email
  - Filter by: Status (Active, Canceled)
  - Search by: Name, Email
  - Export to CSV (name, email, registration_date)
  - Bulk actions: Send reminder email (future feature)
  - Click dancer name → view dancer profile (optional)

  Data Needed:
  // Via new Registration.getByEvent(eventId)
  [
    {
      id: uuid,
      dancer_id: user_id,
      dancer_name: string,
      dancer_email: string,
      event_id: uuid,
      registered_at: ISO datetime,
      status: 'active' | 'cancelled',
      check_in_status: 'pending' | 'checked_in'
    }
  ]