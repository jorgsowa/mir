===description===
With include_vendor_targets=follow an include target under vendor joins the closure
===config===
file_extensions=module,inc
include_seed=a.module
include_vendor_targets=follow
===file:a.module===
<?php
require __DIR__ . '/vendor/pkg/helper.inc';
function a_hook(): int { return pkg_helper(); }
===file:vendor/pkg/helper.inc===
<?php
function pkg_helper(): int { return 1; }
===expect===
