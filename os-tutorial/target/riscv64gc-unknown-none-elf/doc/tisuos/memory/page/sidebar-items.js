initSidebarItems({"constant":[["KERNEL_PAGE_NUM",""],["PAGE_SIZE",""]],"enum":[["PageBit",""]],"fn":[["alloc_kernel_page","申请内核用的页 分配前清零"],["alloc_user_page","申请给定数量的页表，标记为用户 不负责清零"],["clear_page","清理页面 在分配前进行清零操作，速度较慢"],["free_page","释放给定地址对应的页表"],["init","初始化 页表结构体放在 KERNEL_HEAP 起始处，连同 MMIO 区域在内作为保留区域 可分配页表在这之后 此外将页表分为内核与用户两部分，分别用于内核与用户的内存分配"],["print_page_alloc","打印页表分配信息 用于调试"]],"static":[["BSS_END",""],["BSS_START",""],["DATA_END",""],["DATA_START",""],["HEAP_SIZE",""],["HEAP_START",""],["KERNEL_HEAP_START",""],["KERNEL_PAGE_STRUCT_START",""],["KERNEL_STACK_END",""],["KERNEL_STACK_START",""],["MEMORY_END",""],["MEMORY_START",""],["NUM_RESERVE",""],["PAGES",""],["RODATA_END",""],["RODATA_START",""],["TEXT_END",""],["TEXT_START",""],["TOTAL_PAGE",""],["USER_HEAP_START",""],["USER_PAGE_NUM",""],["USER_PAGE_STRUCT_START",""]],"struct":[["Page",""]]});