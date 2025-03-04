#include <vector>
#include <string>
#include <iostream>

std::vector<std::string> split(std::string s, const std::string& d) {
    std::vector<std::string> result;
    size_t pos = 0;
    std::string token;
    while ((pos = s.find(d)) != std::string::npos) {
        token = s.substr(0, pos);
        result.push_back(token);
        s.erase(0, pos + d.length());
    }
    result.push_back(s); // 添加最后一个子字符串
    return result;
}


int main() {
    std::vector<std::string> result = split("1,2,3", ",");
    for (const auto& str : result) {
        std::cout << str << " ";
    }
    return 0;
}