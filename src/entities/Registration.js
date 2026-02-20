import apiClient from '@/integrations/Core';

export class Registration {
  static async list() {
    return apiClient.get('/registrations');
  }

  static async getByEvent(eventId) {
    return apiClient.get(`/registrations?event_id=${eventId}`);
  }

  static async create(registrationData) {
    return apiClient.post('/registrations', registrationData);
  }
}
