import { defineCollection, z } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: docsSchema({
      extend: z.object({
        // Stable MIR#### identifier for issue-kind reference pages.
        // Surfaced globally by the PageTitle override.
        code: z.string().optional(),
      }),
    }),
  }),
};
