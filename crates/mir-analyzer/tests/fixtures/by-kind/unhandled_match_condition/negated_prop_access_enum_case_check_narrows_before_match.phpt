===description===
`$this->status === Status::Pending` narrowing the false branch by excluding
that case from the property's own tracked type — property-access
counterpart of negated_enum_case_check_narrows_before_match.phpt.
===file===
<?php
enum Status { case Active; case Inactive; case Pending; }

class Job {
    public Status $status;

    public function __construct(Status $status) {
        $this->status = $status;
    }

    public function label(): string {
        if ($this->status === Status::Pending) {
            return 'p';
        }
        return match ($this->status) {
            Status::Active => 'a',
            Status::Inactive => 'i',
        };
    }
}
===expect===
