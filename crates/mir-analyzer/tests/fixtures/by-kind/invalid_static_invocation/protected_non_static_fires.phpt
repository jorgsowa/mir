===description===
InvalidStaticInvocation fires for a protected non-static method; visibility does not suppress it.
===file===
<?php
class Service {
    protected function build(): void {}
}

Service::build();
===expect===
InvalidStaticInvocation@6:0-6:16: Non-static method Service::build() cannot be called statically
