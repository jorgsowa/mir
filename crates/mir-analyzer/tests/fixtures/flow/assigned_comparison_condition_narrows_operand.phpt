===description===
`if ($isNew = null === $h)` narrows `$h` like `if (null === $h)`
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class H {
    public static function get(?self $h = null): self {
        if ($isNew = null === $h) {
            $h = new self();
        }
        return $h;
    }
}
