import apiClient from '@/integrations/Core';

export class Event {
  static async list(sort = '-start_date', limit = 50) {
    return apiClient.get(`/events?sort=${sort}&limit=${limit}`);
  }

  static async filter(filters, sort = '-start_date', limit = 50) {
    const params = new URLSearchParams({
      sort,
      limit,
      ...filters
    });
    return apiClient.get(`/events?${params}`);
  }

  static async get(id) {
    return apiClient.get(`/events/${id}`);
  }

  static async create(eventData) {
    return apiClient.post('/events', eventData);
  }

  static async update(id, eventData) {
    return apiClient.put(`/events/${id}`, eventData);
  }

  static async delete(id) {
    return apiClient.delete(`/events/${id}`);
  }

  static async getByOrganizer(organizerId) {
    return apiClient.get(`/events?organizer_id=${organizerId}`);
  }
}
