import { get, post } from './client';
import type { CreatedRecord, ForumTopic, ForumTopicDetail } from './types';

export function listTopics(): Promise<ForumTopic[]> {
  return get<ForumTopic[]>('/forum/topics');
}

export function getTopic(id: number): Promise<ForumTopicDetail> {
  return get<ForumTopicDetail>(`/forum/topics/${id}`);
}

export function createTopic(title: string, content: string): Promise<CreatedRecord> {
  return post<CreatedRecord>('/forum/topics', { title, content });
}

export function replyToTopic(id: number, content: string): Promise<CreatedRecord> {
  return post<CreatedRecord>(`/forum/topics/${id}/posts`, { content });
}
