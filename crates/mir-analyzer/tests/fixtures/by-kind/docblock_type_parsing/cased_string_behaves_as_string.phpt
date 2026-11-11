===description===
A `lowercase-string` is a string: it may be falsy, compared with literals, concatenated and passed on as a string.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesString(string $value): void {}

/** @param non-empty-string $value */
function takesNonEmpty(string $value): void {}

/** @param lowercase-string $tag */
function describe(string $tag): string {
    if ($tag === '') {
        return 'none';
    }
    if (!$tag) {
        return 'zero';
    }
    if ($tag === 'draft') {
        /** @mir-check $tag is "draft" */
        return $tag;
    }
    takesString($tag);
    $lookup = [$tag => strlen($tag)];
    return $tag . '!' . ($lookup[$tag] ?? 0);
}

/** @param non-empty-lowercase-string $tag */
function forward(string $tag): void {
    takesNonEmpty($tag);
    takesString($tag);
    /** @mir-check $tag is non-empty-lowercase-string */
    echo $tag;
}
