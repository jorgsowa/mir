===description===
`$this->status === Status::Active` narrows the property itself, not just a
plain-variable receiver — a subsequent exhaustive `match ($this->status)`
inside the guard must see only the narrowed case, matching the existing
plain-variable enum-case narrowing.
===file===
<?php
enum Status { case Active; case Inactive; }

class Job {
    public Status $status = Status::Active;

    public function run(): string {
        if ($this->status === Status::Active) {
            return match ($this->status) {
                Status::Active => 'active',
            };
        }
        return 'other';
    }
}
===expect===
