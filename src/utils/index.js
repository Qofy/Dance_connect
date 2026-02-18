export const createPageUrl = (pageName) => {
  const pageMap = {
    Dashboard: "/dashboard",
    Today: "/today",
    Calendar: "/calendar",
    MapView: "/map",
    Profile: "/profile",
    CreateEvent: "/create-event",
    Register: "/register",
    AdminEventManager: "/admin/events",
    UserManager: "/admin/users",
    EventProfile: "/event"
  };
  return pageMap[pageName] || "/";
};

export const formatDate = (date) => {
  return new Date(date).toLocaleDateString();
};

export const formatPrice = (price) => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD'
  }).format(price);
};
