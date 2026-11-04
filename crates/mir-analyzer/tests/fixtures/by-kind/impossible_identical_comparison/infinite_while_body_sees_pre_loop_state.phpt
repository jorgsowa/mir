===description===
A while (true) body still sees the pre-loop state on later passes
===file===
<?php
function f(\Iterator $it, ?string $last = null): void {
    while (true) {
        if (null !== $last) {
            $it->next();
        }
        if (!$it->valid()) {
            break;
        }
        $last = 'x';
    }
}
