===description===
Inside the function a mask over another class is used as an int: bit tests, comparisons and arithmetic are accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Mode {
    public const OPEN = 1;
    public const APPEND = 2;
}

function takesInt(int $value): void {}

/** @param int-mask-of<Mode::*> $mode */
function open(int $mode): string {
    if (($mode & Mode::APPEND) !== 0) {
        return 'a';
    }
    if ($mode === 0) {
        return 'none';
    }
    takesInt($mode);
    takesInt($mode | Mode::OPEN);
    return 'r' . ($mode + 1);
}
