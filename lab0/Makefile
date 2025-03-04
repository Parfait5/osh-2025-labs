CXX = g++
CXXFLAGS = -Wall -Wextra -std=c++11
TARGET = bubble_sort
SRCS = main.cpp bubblesort.cpp
OBJS = $(SRCS:.cpp=.o)

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CXX) $(CXXFLAGS) -o $@ $^

%.o: %.cpp bubblesort.hpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)