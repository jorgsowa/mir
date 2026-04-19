===source===
<?php
namespace MyApp {
    class Service {
        public function handle(): void {
            nonexistent_function();
        }
    }
}
===expect===
UndefinedFunction: Function nonexistent_function() is not defined
