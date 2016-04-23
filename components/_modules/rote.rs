use lua;
use modules::ModuleTable;
use runtime::Runtime;
use runner::Runner;


pub const MTABLE: ModuleTable = ModuleTable(&[
    ("create_task", create_task),
    ("create_rule", create_rule),
    ("set_default_task", set_default_task),
]);
