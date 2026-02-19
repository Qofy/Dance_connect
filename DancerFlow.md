creating a Dancer User Flow Spec for login/register and first experience:

  ---
  🩰 Dancer User Journey — Login → First Experience

  Phase 1: Authentication

  Login Page (/login)

  When: New or returning dancer
  Goal: Authenticate with email + password

  UI Flow:
  ┌─────────────────────────────────────┐
  │   DANCECONNECT                      │
  │   BRUTAL DANCE EVENTS               │
  │                                     │
  │   EMAIL                             │
  │   [________________]                │
  │                                     │
  │   PASSWORD                          │
  │   [________________]                │
  │                                     │
  │   [ LOGIN ]                         │
  │                                     │
  │   Don't have an account?            │
  │   Register here                     │
  └─────────────────────────────────────┘

  Data Collected: email, password
  Success Path: → Dashboard
  Error Handling: "Invalid credentials" / "Account not found"

  ---
  Registration Page (/register)

  When: New dancer
  Goal: Create account with role selection

  Step 1: Basic Info
  ┌─────────────────────────────────────┐
  │   CREATE ACCOUNT                    │
  │                                     │
  │   NAME                              │
  │   [________________]                │
  │                                     │
  │   EMAIL                             │
  │   [________________]                │
  │                                     │
  │   PASSWORD                          │
  │   [________________]                │
  │                                     │
  │   ROLE (required)                   │
  │   ○ DANCER (I want to join events)  │
  │   ○ CREATOR (I organize events)     │
  │   ○ BOTH                            │
  │                                     │
  │   [ NEXT ]                          │
  └─────────────────────────────────────┘

  Data Collected: name, email, password, role

  ---
  Phase 2: Dancer Onboarding (if role = "dancer" or "both")

  After Step 1 → Automatic redirect to Dance Profile Setup

  Dance Preferences (/register continued, modal or new page)

  Step 2: Dance Styles Selection
  ┌─────────────────────────────────────┐
  │   SELECT YOUR DANCE STYLES          │
  │                                     │
  │   Pick at least 1 (optional max 5)  │
  │                                     │
  │   [SALSA]  [BACHATA]  [KIZOMBA]    │
  │   [SWING]  [TANGO]    [HIP-HOP]    │
  │   [CONTEMPORARY] [BALLROOM]        │
  │   [JAZZ]   [LATIN]    [HOUSE]      │
  │                                     │
  │   [ SKIP ]  [ NEXT ]               │
  └─────────────────────────────────────┘

  Data Collected: dance_styles[]
  Behavior:
  - User can skip (defaults to empty array)
  - Used for filtering dashboard events
  - Can edit later in profile

  ---
  Step 3: Location Setup
  ┌─────────────────────────────────────┐
  │   WHERE DO YOU DANCE?               │
  │                                     │
  │   📍 DETECT MY LOCATION             │
  │   ↓                                 │
  │   San Francisco, CA                 │
  │   [✓] Use this location             │
  │                                     │
  │   OR ENTER MANUALLY:                │
  │   CITY:     [_____________]         │
  │   STATE:    [_____________]         │
  │   ZIP:      [_____________]         │
  │                                     │
  │   [ SKIP ]  [ COMPLETE ]           │
  └─────────────────────────────────────┘

  Data Collected: city, state, zip, latitude, longitude
  Behavior:
  - Geolocation auto-detects on first load
  - User can override manually
  - Skippable (dashboard shows national events instead)

  ---
  Step 4: Confirmation & Account Created
  ┌─────────────────────────────────────┐
  │   🎉 WELCOME TO DANCECONNECT        │
  │                                     │
  │   Your profile is ready!            │
  │                                     │
  │   NAME: Alex                        │
  │   ROLE: DANCER                      │
  │   STYLES: Salsa, Bachata           │
  │   LOCATION: San Francisco, CA      │
  │                                     │
  │   [ GO TO DASHBOARD ]              │
  └─────────────────────────────────────┘

  ---
  Phase 3: First Experience on Dashboard

  What a newly-registered dancer sees:

  Hero Section

  ┌────────────────────────────────────────┐
  │  🎭 WELCOME TO THE DANCE SCENE         │
  │                                        │
  │  BRUTAL EVENTS • EPIC MOVES • ENERGY   │
  │                                        │
  │  📍 Showing events in San Francisco    │
  └────────────────────────────────────────┘

  Quick Actions (Prominent)

  ┌─────────────────┬─────────────────┬─────────────────┐
  │  CALENDAR VIEW  │  MAP VIEW       │  NEARBY EVENTS  │
  │  Browse by date │  See by location│  Join tonight   │
  └─────────────────┴─────────────────┴─────────────────┘

  Platform Stats

  ┌──────────────┬──────────────┬──────────────┬──────────────┐
  │ 247 EVENTS   │ 42 CITIES    │ 3,562 DANCERS│ ENERGY LEVEL │
  │ This month   │ Connected    │ Registered   │ MAXIMUM ⚡   │
  └──────────────┴──────────────┴──────────────┴──────────────┘

  Search & Filter

  Search box: "SEARCH BRUTAL DANCE EVENTS..."
  Filter dropdown: "ALL STYLES" (auto-populated with their selected styles)

  Event Cards (Filtered to Their Location + Styles)

  For each event:
  ┌────────────────────────────────────┐
  │ [EVENT IMAGE or COLOR BLOCK]       │
  │                                    │
  │ SALSA NIGHT AT THE MISSION         │
  │ 📅 Mar 1, 2026                     │
  │ 📍 San Francisco, CA               │
  │ 👥 42 DANCERS REGISTERED           │
  │ 💰 FREE                            │
  │                                    │
  │ Tags: [SALSA] [SOCIAL]            │
  │                                    │
  │ [ REGISTER ]  [ DETAILS ]         │
  └────────────────────────────────────┘

  ---
  Phase 4: Key User Actions Available

  ┌───────────────────────┬───────────────────────────┬────────────────────────────────────────────────────────┐
  │        Action         │           Page            │                        Outcome                         │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Register for Event    │ Event Card → Event Detail │ Added to registration list, gets event updates         │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ View Event Details    │ Event Detail              │ Full info: time, address, styles, attendees, rules     │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Search Events         │ Dashboard                 │ Filter by keyword                                      │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Filter by Style       │ Dashboard                 │ Shows only registered styles (if any)                  │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ View Calendar         │ /calendar                 │ Browse events by date, click to register               │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ View Map              │ /map                      │ See events by location, click to register              │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Edit Profile          │ /edit-profile             │ Update dance styles, location, personal info           │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ View My Registrations │ /profile                  │ See "Registered Events" section with all joined events │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Report Bug            │ Floating Button           │ Send feedback with screenshot                          │
  ├───────────────────────┼───────────────────────────┼────────────────────────────────────────────────────────┤
  │ Suggest Feature       │ Floating Button           │ Submit feature request                                 │
  └───────────────────────┴───────────────────────────┴────────────────────────────────────────────────────────┘

  ---
  Phase 5: Data Model for Dancer

  // User object when role = "dancer"
  {
    id: "uuid",
    email: "alex@example.com",
    full_name: "Alex",
    role: "user",  // Role-based access (user vs admin)
    user_type: "dancer",  // Persona: dancer | creator | both

    // Dance profile
    dance_styles: ["salsa", "bachata"],

    // Location
    city: "San Francisco",
    state: "CA",
    zip_code: "94105",
    latitude: 37.7749,
    longitude: -122.4194,

    // Engagement
    registered_events: ["event_1", "event_2"],
    created_at: "2026-02-19T00:00:00Z",
    updated_at: "2026-02-19T00:00:00Z"
  }

  ---
  Phase 6: Edge Cases & Behaviors

  ┌─────────────────────────────────┬────────────────────────────────────────────────────────────────────────────────┐
  │            Scenario             │                                    Behavior                                    │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Dancer skips all optional steps │ → Dashboard with ALL events (national)                                         │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Dancer selects no dance styles  │ → Can still register for any event                                             │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Dancer denies geolocation       │ → Manual entry required, can skip for national view                            │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Email already registered        │ → "Email exists. Login here." link                                             │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Weak password                   │ → "Must be 8+ chars, include number & symbol"                                  │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ No events in their area         │ → "No events in San Francisco yet. Browse national events or join waitlist"    │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Dancer clicks event card        │ → Full details page with "REGISTER" CTA                                        │
  ├─────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ Dancer registers for event      │ → Toast: "✓ Registered! Check your email for details" → Event added to profile │
  └─────────────────────────────────┴────────────────────────────────────────────────────────────────────────────────┘

  ---
  Phase 7: Success Metrics

  A newly-registered dancer should:
  - ✓ See events relevant to their location (if provided)
  - ✓ See events matching their dance styles (if provided)
  - ✓ Understand how to register for an event (prominent button)
  - ✓ Be able to navigate to their profile to see registered events
  - ✓ Know where to give feedback (bug/wish buttons)
  - ✓ Complete the entire flow in <5 minutes

  ---
  Summary: What Dancers See & Do

  LOGIN/REGISTER → PROFILE SETUP → DASHBOARD → BROWSE → REGISTER → PROFILE
       ↓               ↓               ↓          ↓        ↓          ↓
    Email+Pass    Styles+Location   Welcome!   Search   Confirm   My Events
    (2 mins)      (2 mins)           Page    Filter    Event     View Regs

# Implemented#
What changed / was created
register.svelte — Replaced the role dropdown with styled radio buttons (DANCER / CREATOR / BOTH). After successful registration, dancers and "both" users are redirected to the onboarding flow instead of the dashboard. Better error messages for duplicate email and weak password.

onboarding/dance-styles.svelte (new) — Step 2. Uses the existing DanceStylePicker component, enforces max 5 selections, shows a live preview of selected styles. SKIP or NEXT both advance to location step, saving selections to localStorage.

onboarding/location.svelte (new) — Step 3. Geolocation "DETECT MY LOCATION" button with a confirm banner. Manual city/state/zip fallback. SKIP sends dancer directly to welcome.

onboarding/welcome.svelte (new) — Step 4. Shows a profile summary card (name, role, styles, location). On "GO TO DASHBOARD", saves dance styles + location to the backend via PUT /users/{id}, sets is_new_dancer in localStorage, then redirects.

dashboard.svelte — Dancer-personalized experience:

Green welcome banner on first arrival (dismissible)
Hero title says WELCOME, [NAME]! for new dancers
Style chips in the hero auto-populated from dancer's saved styles — click one to instantly filter
Style dropdown marks dancer's own styles with ★
Active filter chip with a clear button
"No events in [city]" empty state with a "Browse national events" escape hatch
edit-profile.svelte — Added a full Dance Styles section (using DanceStylePicker) and a Location section (city/state/zip). Both sections persist to the API on save.