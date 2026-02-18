// @ts-nocheck


export const routes = {
  "meta": {},
  "id": "_default",
  "name": "",
  "file": {
    "path": "src/pages",
    "dir": "src",
    "base": "pages",
    "ext": "",
    "name": "pages"
  },
  "rootName": "default",
  "routifyDir": import.meta.url,
  "children": [
    {
      "meta": {},
      "id": "_default_admin",
      "name": "admin",
      "module": false,
      "file": {
        "path": "src/pages/admin",
        "dir": "src/pages",
        "base": "admin",
        "ext": "",
        "name": "admin"
      },
      "children": [
        {
          "meta": {},
          "id": "_default_admin_events_svelte",
          "name": "events",
          "file": {
            "path": "src/pages/admin/events.svelte",
            "dir": "src/pages/admin",
            "base": "events.svelte",
            "ext": ".svelte",
            "name": "events"
          },
          "asyncModule": () => import('../src/pages/admin/events.svelte'),
          "children": []
        },
        {
          "meta": {},
          "id": "_default_admin_users_svelte",
          "name": "users",
          "file": {
            "path": "src/pages/admin/users.svelte",
            "dir": "src/pages/admin",
            "base": "users.svelte",
            "ext": ".svelte",
            "name": "users"
          },
          "asyncModule": () => import('../src/pages/admin/users.svelte'),
          "children": []
        }
      ]
    },
    {
      "meta": {},
      "id": "_default_calendar_svelte",
      "name": "calendar",
      "file": {
        "path": "src/pages/calendar.svelte",
        "dir": "src/pages",
        "base": "calendar.svelte",
        "ext": ".svelte",
        "name": "calendar"
      },
      "asyncModule": () => import('../src/pages/calendar.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_create_event_svelte",
      "name": "create-event",
      "file": {
        "path": "src/pages/create-event.svelte",
        "dir": "src/pages",
        "base": "create-event.svelte",
        "ext": ".svelte",
        "name": "create-event"
      },
      "asyncModule": () => import('../src/pages/create-event.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_dashboard_svelte",
      "name": "dashboard",
      "file": {
        "path": "src/pages/dashboard.svelte",
        "dir": "src/pages",
        "base": "dashboard.svelte",
        "ext": ".svelte",
        "name": "dashboard"
      },
      "asyncModule": () => import('../src/pages/dashboard.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_edit_profile_svelte",
      "name": "edit-profile",
      "file": {
        "path": "src/pages/edit-profile.svelte",
        "dir": "src/pages",
        "base": "edit-profile.svelte",
        "ext": ".svelte",
        "name": "edit-profile"
      },
      "asyncModule": () => import('../src/pages/edit-profile.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_event_svelte",
      "name": "event",
      "file": {
        "path": "src/pages/event.svelte",
        "dir": "src/pages",
        "base": "event.svelte",
        "ext": ".svelte",
        "name": "event"
      },
      "asyncModule": () => import('../src/pages/event.svelte'),
      "children": []
    },
    {
      "meta": {
        "isDefault": true
      },
      "id": "_default_index_svelte",
      "name": "index",
      "file": {
        "path": "src/pages/index.svelte",
        "dir": "src/pages",
        "base": "index.svelte",
        "ext": ".svelte",
        "name": "index"
      },
      "asyncModule": () => import('../src/pages/index.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_login_svelte",
      "name": "login",
      "file": {
        "path": "src/pages/login.svelte",
        "dir": "src/pages",
        "base": "login.svelte",
        "ext": ".svelte",
        "name": "login"
      },
      "asyncModule": () => import('../src/pages/login.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_map_svelte",
      "name": "map",
      "file": {
        "path": "src/pages/map.svelte",
        "dir": "src/pages",
        "base": "map.svelte",
        "ext": ".svelte",
        "name": "map"
      },
      "asyncModule": () => import('../src/pages/map.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_profile_svelte",
      "name": "profile",
      "file": {
        "path": "src/pages/profile.svelte",
        "dir": "src/pages",
        "base": "profile.svelte",
        "ext": ".svelte",
        "name": "profile"
      },
      "asyncModule": () => import('../src/pages/profile.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_register_svelte",
      "name": "register",
      "file": {
        "path": "src/pages/register.svelte",
        "dir": "src/pages",
        "base": "register.svelte",
        "ext": ".svelte",
        "name": "register"
      },
      "asyncModule": () => import('../src/pages/register.svelte'),
      "children": []
    },
    {
      "meta": {},
      "id": "_default_today_svelte",
      "name": "today",
      "file": {
        "path": "src/pages/today.svelte",
        "dir": "src/pages",
        "base": "today.svelte",
        "ext": ".svelte",
        "name": "today"
      },
      "asyncModule": () => import('../src/pages/today.svelte'),
      "children": []
    },
    {
      "meta": {
        "dynamic": true,
        "dynamicSpread": true,
        "order": false,
        "inline": false
      },
      "name": "[...404]",
      "file": {
        "path": ".routify/components/[...404].svelte",
        "dir": ".routify/components",
        "base": "[...404].svelte",
        "ext": ".svelte",
        "name": "[...404]"
      },
      "asyncModule": () => import('./components/[...404].svelte'),
      "children": []
    }
  ]
}
export default routes