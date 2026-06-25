===description===
Assigning to $this->prop inside a @psalm-external-mutation-free method is
allowed — external-mutation-free only blocks mutations to external parameters.
===file===
<?php

class Updater {
    public string $state = 'idle';

    /** @psalm-external-mutation-free */
    public function activate(): void {
        $this->state = 'active';
    }
}
===expect===
