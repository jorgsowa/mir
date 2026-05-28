===description===
Inside a generic class, properties NOT typed with class templates must still trigger
InvalidPropertyAssignment for wrong concrete types
===file===
<?php

/**
 * @template A
 */
class Box {
    private string $label = '';

    public function bad(int $n): void {
        $this->label = $n;
    }
}
===expect===
InvalidPropertyAssignment@10:9: Property $label expects 'string', cannot assign 'int'
