===description===
`self::A` in a tuple shape is an item type, not a key named `self`, so the matching tuple is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Strategy {
    public const SYMLINK = 10;
    public const MIRROR = 20;

    /** @return array{self::SYMLINK|self::MIRROR, non-empty-list<self::SYMLINK|self::MIRROR>} */
    public function pick(bool $mirror): array {
        $current = $mirror ? self::MIRROR : self::SYMLINK;
        return [$current, [$current]];
    }
}
