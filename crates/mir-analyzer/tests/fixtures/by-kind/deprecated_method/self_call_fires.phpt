===description===
DeprecatedMethod fires even when calling a deprecated method via $this within the same class — there is no within-class exemption.
===file===
<?php
class Service {
    /** @deprecated use newProcess() instead */
    public function process(): void {}

    public function run(): void {
        $this->process();
    }
}
===expect===
DeprecatedMethod@7:8-7:24: Method Service::process() is deprecated: use newProcess() instead
