===description===
A literal with an uppercase letter is not a `lowercase-string`; digits, symbols and the empty string are.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param lowercase-string $slug */
function findBySlug(string $slug): void {}

findBySlug('blog-post');
findBySlug('2024');
findBySlug('');
findBySlug('Blog-Post');
//         ^^^^^^^^^^^ InvalidArgument: Argument $slug of findBySlug() expects 'lowercase-string', got '"Blog-Post"'
findBySlug('blog-posT');
//         ^^^^^^^^^^^ InvalidArgument: Argument $slug of findBySlug() expects 'lowercase-string', got '"blog-posT"'
findBySlug('BLOG');
//         ^^^^^^ InvalidArgument: Argument $slug of findBySlug() expects 'lowercase-string', got '"BLOG"'
