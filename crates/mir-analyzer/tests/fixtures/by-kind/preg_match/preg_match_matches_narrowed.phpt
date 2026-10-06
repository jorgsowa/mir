===description===
FP-D: preg_match() sets $matches by reference. After a successful match,
$matches should be typed as array — not mixed. Using $matches[0] after
preg_match must not emit UndefinedVariable or NonExistentArrayOffset.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function extract(string $input): string {
    if (preg_match('/(\d+)/', $input, $matches)) {
        return $matches[0];
    }
    return '';
}
