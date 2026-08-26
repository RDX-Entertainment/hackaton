export type WikiSummary = {
  slug: string;
  title: string;
  updated_at: string;
};

export type WikiPage = WikiSummary & {
  content: string;
  author: string | null;
  created_at: string;
};

export type ForumTopic = {
  id: number;
  title: string;
  author: string;
  posts: number;
  updated_at: string;
};

export type ForumPost = {
  id: number;
  content: string;
  author: string;
  created_at: string;
};

export type ForumTopicDetail = {
  id: number;
  title: string;
  author: string;
  created_at: string;
  posts: ForumPost[];
};

