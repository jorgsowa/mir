===description===
InvalidStaticInvocation fires for a protected non-static method; visibility does not suppress it.
===file===
<?php
class Service {
    protected function build(): void {}
}

Service::build();
//<^^^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Service::build() cannot be called statically
