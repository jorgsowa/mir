===description===
namespaced method body
===file===
<?php
namespace MyApp {
    class Service {
        public function handle(): void {
            nonexistent_function();
        }
    }
}
===expect===
UndefinedFunction@5:13-5:35: Function nonexistent_function() is not defined
