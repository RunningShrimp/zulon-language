const integer_types: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
]; // 整数类型
const float_types: &[&str] = &[
    "f32", "f64",
];// 浮点数类型
const boolean_type: &str = "bool"; // 布尔类型，true 或 false
const char_type: &str = "char"; // 字符类型，表示单个 Unicode 字符, ASCII 范围内的字符可以和i8类型互换
const string_type: &str = "str"; // 字符串类型，表示一段 UTF-8 编码的文本
const tensor_type: &str = "tensor"; // 张量类型，表示多维数组，通常用于机器学习和科学计算
const complex_number_type: &str = "complex"; // 复数类型，表示具有实部和虚部的数,a + bi 形式
const pointer_type: &str = "pointer"; // 指针类型，表示内存地址
const array_type: &str = "array"; // 数组类型，表示固定大小的元素集合
const slice_type: &str = "slice"; // 切片类型，表示动态大小的元素集合
const tuple_type: &str = "tuple"; // 元组类型，表示多个不同类型的值的组合
const function_type: &str = "function"; // 函数类型，表示可调用的代码块
const option_type: &str = "option"; // 可选类型，表示值可能存在或不存在
const result_type: &str = "result"; // 结果类型，表示操作可能成功或失败，?运算符或!运算符常用到
const unit_type: &str = "()"; // 单元类型，表示没有值，类似于 void类型