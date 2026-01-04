export const getStaffGroups = function() {
    return [
        {
            name: "kitchen",
            staffList: [
                {
                    name: "Alice",
                    id: 0
                },
                {
                    name: "Bob",
                    id: 1
                }, 
                {
                    name: "Charlie",
                    id: 2
                }
            ],
        },

        {
            name: "hall",
            staffList: [
                {
                    name: "Dave",
                    id: 3
                },
                {
                    name: "Eve",
                    id: 4
                },
            ],
        },
    ];
};

export const getWeekRules = function () {
    return {

        mon: {
            morning: [
                {
                    groupId: 0, // シフトホールに入りうる職能
                    index: 1,    // index ()
                }
            ],
            afternoon: [
            ],
        },

        tue: {
            morning: [
            ],
            afternoon: [
            ],
        },

        wed: {
            morning: [
            ],
            afternoon: [
            ],
        },

        thu: {
            morning: [
            ],
            afternoon: [
            ],
        },

        fri: {
            morning: [
            ],
            afternoon: [
            ],
        },

        sat: {
            morning: [
            ],
            afternoon: [
            ],
        },

        sun: {
            morning: [
            ],
            afternoon: [
            ],
        },

    };
}
