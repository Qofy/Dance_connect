import apiClient from '@/integrations/Core';

export class User {
  static async me() {
    return apiClient.get('/users/me');
  }

  static async list() {
    return apiClient.get('/users');
  }

  static async get(id) {
    return apiClient.get(`/users/${id}`);
  }

  static async create(userData) {
    return apiClient.post('/users', userData);
  }

  static async update(id, userData) {
    return apiClient.put(`/users/${id}`, userData);
  }
}
