===description===
Negated instanceof guard narrows type after early return (PostVoter pattern)
===file===
<?php
class Post {
    public function getTitle(): string { return ''; }
}
class Comment {}

/**
 * @param Post|Comment $subject
 */
function test_union(Post|Comment $subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    /** @mir-check $subject is Post */
    $_ = $subject;
}

/**
 * @param mixed $subject
 */
function test_mixed($subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    /** @mir-check $subject is Post */
    $_ = $subject;
}

function test_object(object $subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    /** @mir-check $subject is Post */
    $_ = $subject;
}
===expect===
