import React from 'react';
import { Link } from 'react-router-dom';

const FriendList: React.FC = () => {
    const friends = [
        { id: 1, name: 'John Doe' },
        { id: 2, name: 'Jane Smith' },
        { id: 3, name: 'Bob Johnson' },
    ];

    return (
        <div className="friend-list">
            <h3>好友列表</h3>
            <ul>
                {friends.map(friend => (
                    <li key={friend.id}>
                        <Link to={`/home/chat/${friend.id}`}>{friend.name}</Link> {/* 修正路径 */}
                    </li>
                ))}
            </ul>
        </div>

    );
};

export default FriendList;
